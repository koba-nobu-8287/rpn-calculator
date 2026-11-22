use std::io::Write;
use std::process::{Command, Output, Stdio};
fn run(args: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rpn-calc"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let _ = child.stdin.take().unwrap().write_all(input);
    child.wait_with_output().unwrap()
}
fn success(args: &[&str], input: &[u8], stdout: &str) {
    let r = run(args, input);
    assert_eq!(r.status.code(), Some(0), "{r:?}");
    assert_eq!(r.stdout, stdout.as_bytes());
    assert!(r.stderr.is_empty());
}
#[test]
fn batch_outputs_only_final_x() {
    success(&["--base", "hex", "hex FF dec 10 +"], b"", "265\n");
    success(
        &["--word-size", "8", "--signed", "unsigned", "FF 01 +"],
        b"",
        "0\n",
    );
    success(&["--float", "keys \"1 . 5\""], b"", "1.5\n");
    success(&["--base", "dec", "-12"], b"", "-12\n");
    success(&["--register-count", "2", "7 sto 1"], b"", "7\n");
    success(&["--register-count", "2", "rcl 1"], b"", "0\n");
}
#[test]
fn batch_error_codes_and_diagnostics() {
    for (args, code) in [
        (vec!["5 0 /"], 1),
        (vec!["5 dblmul"], 1),
        (vec!["--float", "keys \"1 eex\""], 1),
        (vec!["5 unknownword"], 2),
        (vec!["0xG"], 2),
        (vec!["stack"], 2),
        (vec!["--float", "--base", "hex"], 2),
        (vec!["--word-size", "0"], 2),
        (vec!["--register-count", "1025"], 2),
        (vec!["--signed"], 2),
        (vec!["--base", "hex", "--base", "dec"], 2),
        (vec!["1", "2"], 2),
    ] {
        let r = run(&args, b"");
        assert_eq!(r.status.code(), Some(code), "{args:?}: {r:?}");
        assert!(r.stdout.is_empty());
        assert!(!r.stderr.is_empty());
    }
}
#[test]
fn repl_recovers_and_preserves_prior_values() {
    let r = run(&["--base", "dec"], b"5\r\n0 /\nclx\n2 +\nstack\nexit\n99\n");
    assert_eq!(r.status.code(), Some(0));
    assert!(!r.stderr.is_empty());
    let out = String::from_utf8(r.stdout).unwrap();
    assert!(out.contains("X: 7"));
    assert!(!out.contains("99"));
}
#[test]
fn repl_editing_and_eof() {
    let r = run(&["--float"], b"keys \"1 eex\"\nstack\nkeys \"clx\"\n");
    assert_eq!(r.status.code(), Some(0));
    assert!(r.stderr.is_empty());
    let out = String::from_utf8(r.stdout).unwrap();
    assert!(out.contains("1e"));
    assert!(out.trim_end().ends_with('0'));
}

#[test]
fn test_056_stack_query_shows_edit_buffer_on_x_only() {
    let r = run(
        &["--float"],
        b"keys \"1 eex\"\nstack\nkeys \"2\"\nstack\nenter\nstack\nexit\n",
    );
    assert_eq!(r.status.code(), Some(0));
    assert!(r.stderr.is_empty());
    let text = String::from_utf8(r.stdout).unwrap();
    let x_rows: Vec<_> = text.lines().filter(|line| line.starts_with("X:")).collect();
    assert_eq!(x_rows, ["X: 1 [1e]", "X: 100 [1e2]", "X: 100"]);
    assert!(!text.lines().any(|line| (line.starts_with("T:")
        || line.starts_with("Z:")
        || line.starts_with("Y:"))
        && line.contains('[')));
}
#[test]
fn oversize_line_drains_to_newline() {
    let mut input = vec![b' '; 65537];
    input.extend_from_slice(b"7\n2\n");
    let r = run(&[], &input);
    assert_eq!(r.status.code(), Some(0));
    assert_eq!(r.stdout, b"2\n");
    assert!(!r.stderr.is_empty());
}
#[test]
fn invalid_utf8_reports_input_io_failure() {
    let r = run(&[], &[255, b'\n']);
    assert_eq!(r.status.code(), Some(3));
    assert!(!r.stderr.is_empty());
    assert!(r.stdout.is_empty());
}
#[test]
fn help_exits_without_reading_stdin() {
    let r = run(&["--help"], &[255]);
    assert_eq!(r.status.code(), Some(0));
    let help = String::from_utf8(r.stdout).unwrap();
    assert_eq!(
        help.lines().next(),
        Some(format!("rpn-calc {}", rpn_core::PRODUCT_VERSION).as_str())
    );
    for option in [
        "--base",
        "--signed",
        "--float",
        "--register-count",
        "--word-size",
        "--help",
        "--version",
    ] {
        assert!(
            help.lines()
                .any(|line| line.trim_start().starts_with(option)),
            "missing {option}"
        );
    }
    assert!(r.stderr.is_empty());
}

#[test]
fn version_exits_without_reading_stdin() {
    assert_eq!(rpn_core::PRODUCT_VERSION, "v1.0.0.0");
    success(
        &["--version"],
        &[255],
        &format!("rpn-calc {}\n", rpn_core::PRODUCT_VERSION),
    );
}

#[test]
fn repl_accepts_exact_limit_with_crlf_and_final_unterminated_line() {
    let mut input = vec![b' '; 65535];
    input.extend_from_slice(b"2\r\n3 +");
    success(&["--base", "dec"], &input, "2\n5\n");
}
#[test]
fn repl_syntax_error_executes_no_part_of_the_line() {
    let r = run(
        &["--base", "dec"],
        b"3\n7 unknownword\n2 +\n# ignored\nexit\n",
    );
    assert_eq!(r.status.code(), Some(0));
    assert_eq!(r.stdout, b"3\n5\n");
    assert!(!r.stderr.is_empty());
}

#[test]
fn oneszero_batch_arithmetic_and_raw_transfers() {
    for (expression, canonical, raw) in [
        ("01 FE +", "0", "FF"),
        ("FF 00 sub", "0", "FF"),
        ("00 FE mul", "0", "FF"),
        ("FE 02 div", "0", "FF"),
        ("FB 02 rmd", "0", "FF"),
        ("00 chs", "FF", "FF"),
        ("00 not", "FF", "FF"),
        ("FF sto 0 clx rcl 0", "FF", "FF"),
        ("FF abs", "0", "0"),
        ("FF sqrt", "0", "0"),
        ("7F 01 +", "80", "80"),
    ] {
        for (prefix, expected) in [("", canonical), ("oneszero raw", raw)] {
            let line = format!("hex wsize 8 signed ones {prefix} {expression}");
            success(&[&line], b"", &format!("{expected}\n"));
        }
    }
}

#[test]
fn oneszero_repl_persists_and_help_lists_both_modes() {
    let result = run(
        &[],
        b"wsize 8 signed ones oneszero raw\n01 FE +\noneszero canonical\n01 FE +\nhelp\n",
    );
    assert_eq!(result.status.code(), Some(0));
    assert!(result.stderr.is_empty());
    let output = String::from_utf8(result.stdout).unwrap();
    assert!(output.starts_with("0\nFF\nFF\n0\n"), "{output}");
    assert!(output.contains("oneszero canonical|raw"), "{output}");
}

#[test]
fn sign_modes_control_decimal_display_and_zero_arithmetic() {
    for (expression, expected) in [
        ("signed twos 0xff dec", "-1"),
        ("signed unsigned 0xff dec", "255"),
        ("signed ones 0xff dec", "-0"),
        ("hex 1 cf 3", "1"),
        ("hex 1 sf 3", "01"),
        ("signed ones FF 01 *", "0"),
        ("signed ones oneszero raw FF 01 *", "FF"),
        ("signed ones oneszero raw oneszero canonical FF 01 *", "0"),
        ("signed ones 00 01 sub", "FE"),
        ("signed ones oneszero raw 00 01 sub", "FE"),
    ] {
        success(
            &["--word-size", "8", expression],
            b"",
            &format!("{expected}\n"),
        );
    }
    for mode in ["canonical", "raw"] {
        let expr = format!("signed ones oneszero {mode} 01 FF /");
        let result = run(&["--word-size", "8", &expr], b"");
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
    }
}

#[test]
fn decimal_literal_display_at_sign_boundaries() {
    for (mode, cases) in [
        (
            "unsigned",
            &[
                ("0", "0"),
                ("255", "255"),
                ("256", "0"),
                ("-1", "255"),
                ("-256", "0"),
            ][..],
        ),
        (
            "twos",
            &[
                ("127", "127"),
                ("128", "-128"),
                ("-128", "-128"),
                ("-129", "127"),
            ],
        ),
        (
            "ones",
            &[
                ("127", "127"),
                ("128", "-127"),
                ("-127", "-127"),
                ("-128", "127"),
            ],
        ),
    ] {
        for &(token, display) in cases {
            success(
                &["--base", "dec", "--word-size", "8", "--signed", mode, token],
                b"",
                &format!("{display}\n"),
            );
        }
    }
    for (base, token, display) in [("hex", "-0d123", "85"), ("dec", "0x123", "35")] {
        success(
            &["--base", base, "--word-size", "8", token],
            b"",
            &format!("{display}\n"),
        );
    }
}
