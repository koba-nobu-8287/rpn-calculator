use rpn_cli::{
    LiteralError,
    line::{CliSession, LineError, RunControl},
};
use rpn_core::*;
use rpn_input::*;
fn new_session() -> CliSession {
    CliSession::new(Calculator::new())
}

#[test]
fn keys_keep_successful_prefix_and_resume_across_lines() {
    let mut s = new_session();
    let mut out = Vec::new();
    assert!(matches!(
        s.run_line("bin keys \"1 2\"", &mut out, true),
        Err(LineError::Input(InputError::InvalidDigit {
            digit: '2',
            radix: Radix::Binary
        }))
    ));
    let before = s.snapshot();
    assert_eq!(before.input_buffer.as_deref(), Some("1"));
    assert!(matches!(
        s.run_line("10", &mut out, true),
        Err(LineError::Input(InputError::InputInProgress))
    ));
    assert_eq!(s.snapshot(), before);
    s.run_line(
        "keys \"0 enter\" hex sto 0 keys \"E enter\" rcl 0 +",
        &mut out,
        true,
    )
    .unwrap();
    assert_eq!(
        s.snapshot().calculator.stack[0]
            .as_integer()
            .unwrap()
            .bits(),
        16
    );
}
#[test]
fn incomplete_float_and_calculation_errors_keep_edits() {
    let mut s = new_session();
    let mut out = Vec::new();
    s.run_line("float keys \"1 eex chs\"", &mut out, true)
        .unwrap();
    let before = s.snapshot();
    assert!(matches!(
        s.run_line("div", &mut out, true),
        Err(LineError::Input(InputError::IncompleteEntry))
    ));
    assert_eq!(s.snapshot(), before);
    s.run_line("keys \"2 enter\"", &mut out, true).unwrap();
    assert_eq!(s.snapshot().calculator.stack[0], Number::Float(0.01));
    assert!(matches!(
        s.run_line("1e-", &mut out, true),
        Err(LineError::Literal(LiteralError::InvalidNumber))
    ));
}
#[test]
fn queries_use_core_formatting_and_stop_on_exit() {
    let mut s = new_session();
    let mut out = Vec::new();
    assert_eq!(
        s.run_line("hex FF stack bit? 7 status exit 5", &mut out, true)
            .unwrap(),
        RunControl::Exit
    );
    let printed = String::from_utf8(out).unwrap();
    assert!(printed.starts_with("T: 0\nZ: 0\nY: 0\nX: FF\n1\n"));
    for field in [
        "radix=Hexadecimal",
        "width=16",
        "F5=0",
        "phase=Idle",
        "registers=16",
    ] {
        assert!(printed.contains(field));
    }
}
#[test]
fn line_byte_limit_comments_and_io_errors() {
    let mut s = new_session();
    let mut out = Vec::new();
    s.run_line(&" ".repeat(65536), &mut out, true).unwrap();
    assert!(matches!(
        s.run_line(&" ".repeat(65537), &mut out, true),
        Err(LineError::Syntax(_))
    ));
    let before = s.snapshot();
    assert!(matches!(
        s.run_line("1 keys \"2 # hidden\"", &mut out, true),
        Err(LineError::Syntax(_))
    ));
    assert_eq!(s.snapshot(), before);
    s.run_line("dec 7 # keys \"unterminated", &mut out, true)
        .unwrap();
    assert!(matches!(
        s.run_line("stack", &mut &mut [0u8; 0][..], true),
        Err(LineError::Io(_))
    ));
}
