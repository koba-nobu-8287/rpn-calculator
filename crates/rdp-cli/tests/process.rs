use std::io::Write;
use std::process::{Command, Output, Stdio};
fn run(args: &[&str], input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rdp-calc"))
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
    assert!(String::from_utf8(r.stdout).unwrap().contains("--word-size"));
    assert!(r.stderr.is_empty());
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
