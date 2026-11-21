use super::*;

#[test]
fn corrupt_entry_precedes_commands_keys_and_queries() {
    for (mode, buffer) in [
        (NumericMode::Integer, "1"), // legal digits, but X is zero
        (NumericMode::Integer, "+0"),
        (NumericMode::FloatingPoint, "-0"), // preview sign differs from X
        (NumericMode::FloatingPoint, "0e+0"),
        (NumericMode::FloatingPoint, ".0"),
        (NumericMode::FloatingPoint, "0E0"),
    ] {
        let mut core = Calculator::new();
        core.execute(Command::SetNumericMode(mode)).unwrap();
        let mut session = HpInputSession::new(core);
        session.buffer = buffer.into();
        let before = session.snapshot();
        let expected = SessionError::Input(InputError::InvariantViolation);
        assert_eq!(session.test_bit(255), Err(expected.clone()), "{buffer}");
        for event in [
            InputEvent::Key(Key::Digit('G')),
            InputEvent::Command(Command::ClearX),
        ] {
            assert_eq!(session.apply(event), Err(expected.clone()), "{buffer}");
            assert_eq!(session.snapshot(), before);
        }
    }
}
