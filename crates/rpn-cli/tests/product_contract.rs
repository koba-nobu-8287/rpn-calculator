use rpn_cli::{
    LiteralError,
    line::{CliError, CliSession, LineError},
};
use rpn_core::{CalcError, Calculator};
use rpn_input::{InputError, SessionError};
use std::error::Error;

#[test]
fn semantic_errors_support_generic_rust_error_handling() {
    fn check<E: Error + Clone + Eq>(error: E) {
        assert_eq!(error, error.clone());
        assert!(!error.to_string().is_empty());
    }
    check(InputError::IncompleteEntry);
    check(SessionError::Calculation(CalcError::DivisionByZero));
    check(LiteralError::InvalidNumber);
    check(CliError::MissingArgument("wsize".into()));
    check(rpn_cli::LiteralEntryError::Literal(
        LiteralError::InvalidNumber,
    ));
    let diagnostic = LineError::Input(InputError::IncompleteEntry);
    assert!(diagnostic.source().unwrap().is::<InputError>());
}

#[test]
fn largest_register_and_word_resize_through_cli() {
    use rpn_core::CalculatorConfig;
    let mut s = CliSession::new(
        Calculator::with_config(CalculatorConfig {
            register_count: 1024,
            ..Default::default()
        })
        .unwrap(),
    );
    let mut out = Vec::new();
    s.run_line(
        "hex wsize 64 0x80000000000000FF sto 1023 wsize 8 wsize 64 clx rcl 1023",
        &mut out,
        true,
    )
    .unwrap();
    let before = s.snapshot();
    assert_eq!(before.calculator.registers.len(), 1024);
    assert_eq!(before.calculator.stack[0].as_integer().unwrap().bits(), 255);
    assert_eq!(
        before.calculator.registers[1023],
        before.calculator.stack[0]
    );
    for line in ["sto 1024", "rcl 1024"] {
        assert!(matches!(
            s.run_line(line, &mut out, true),
            Err(LineError::Calculation(CalcError::InvalidRegister(1024)))
        ));
        assert_eq!(s.snapshot(), before);
    }
}

#[test]
fn editing_sign_at_the_256_byte_limit_recovers_on_the_next_key() {
    let mut s = CliSession::new(Calculator::new());
    let mut out = Vec::new();
    s.run_line(
        &format!("float keys \"{}\"", "0".repeat(256)),
        &mut out,
        true,
    )
    .unwrap();
    let full = s.snapshot();
    assert!(matches!(
        s.run_line("chs", &mut out, true),
        Err(LineError::Input(InputError::InputTooLong))
    ));
    assert_eq!(s.snapshot(), full);
    s.run_line("keys \"bs chs\"", &mut out, true).unwrap();
    assert_eq!(
        s.snapshot().input_buffer.unwrap(),
        format!("-{}", "0".repeat(255))
    );
    assert_eq!(
        s.snapshot().calculator.stack[0]
            .as_float()
            .unwrap()
            .to_bits(),
        (-0.0f64).to_bits()
    );
}
