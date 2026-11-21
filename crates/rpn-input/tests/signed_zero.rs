use rpn_core::*;
use rpn_input::*;
// IA-G012 / SI-H003: product decision on 2026-11-21.
#[test]
fn deleting_last_negative_digit_continues_with_positive_zero() {
    for enabled in [false, true] {
        let mut core = Calculator::new();
        core.execute(Command::SetNumericMode(NumericMode::FloatingPoint))
            .unwrap();
        if enabled {
            core.execute(Command::Push(Number::Float(7.0))).unwrap();
        }
        let mut session = HpInputSession::new(core);
        for key in [Key::Digit('1'), Key::ChangeSign] {
            session.apply(InputEvent::Key(key)).unwrap();
        }
        let mut expected = session.snapshot();
        expected.calculator.stack[0] = Number::Float(0.0);
        expected.input_buffer = Some("0".into());
        session.apply(InputEvent::Key(Key::Backspace)).unwrap();
        let actual = session.snapshot();
        assert_eq!(actual, expected);
        assert_eq!(actual.calculator.stack[0].as_float().unwrap().to_bits(), 0);
        session.apply(InputEvent::Key(Key::Digit('2'))).unwrap();
        expected.input_buffer = Some("02".into());
        expected.calculator.stack[0] = Number::Float(2.0);
        assert_eq!(session.snapshot(), expected);
    }
}
