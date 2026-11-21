use rpn_cli::*;
use rpn_core::*;
use rpn_input::*;
fn session() -> HpInputSession {
    HpInputSession::new(
        Calculator::with_config(CalculatorConfig {
            word_config: WordConfig {
                ones_zero_mode: rpn_core::OnesZeroMode::Canonical,
                size: WordSize::new(8).unwrap(),
                signed_mode: SignedMode::TwosComplement,
            },
            ..Default::default()
        })
        .unwrap(),
    )
}
#[test]
fn completed_values_use_current_settings_and_core_push() {
    let mut s = session();
    for event in [
        Command::SetFlag {
            index: 4,
            value: true,
        },
        Command::Store(0),
    ] {
        s.apply(InputEvent::Command(event)).unwrap();
    }
    let mut reference = s.clone();
    for (radix, text, raw) in [
        (Radix::Hexadecimal, "10", 16),
        (Radix::Decimal, "10", 10),
        (Radix::Decimal, "0x123", 35),
        (Radix::Binary, "-", 0),
    ] {
        for target in [&mut s, &mut reference] {
            target
                .apply(InputEvent::Command(Command::SetRadix(radix)))
                .unwrap();
        }
        enter_literal(&mut s, text).unwrap();
        reference
            .apply(InputEvent::Command(Command::Push(Number::Integer(
                Word::from_bits(raw, WordSize::new(8).unwrap()),
            ))))
            .unwrap();
        assert_eq!(s.snapshot(), reference.snapshot());
    }
}
#[test]
fn strict_keys_and_failed_literal_leave_edit_intact() {
    let mut s = session();
    for c in ['F', 'F'] {
        s.apply(InputEvent::Key(Key::Digit(c))).unwrap();
    }
    let before = s.snapshot();
    assert_eq!(
        s.apply(InputEvent::Key(Key::Digit('F'))),
        Err(SessionError::Calculation(CalcError::IntegerOutOfRange))
    );
    assert_eq!(
        enter_literal(&mut s, "0xG"),
        Err(LiteralEntryError::Literal(LiteralError::InvalidDigit {
            digit: 'G',
            radix: Radix::Hexadecimal
        }))
    );
    assert_eq!(s.snapshot(), before);
    assert_eq!(
        enter_literal(&mut s, "0xFFF"),
        Err(LiteralEntryError::Session(SessionError::Input(
            InputError::InputInProgress
        )))
    );
    assert_eq!(s.snapshot(), before);
    s.apply(InputEvent::Command(Command::ClearX)).unwrap();
    enter_literal(&mut s, "0xFFF").unwrap();
    assert_eq!(
        s.snapshot().calculator.stack[0]
            .as_integer()
            .unwrap()
            .bits(),
        255
    );
}
#[test]
fn mode_changes_and_negative_zero_survive_push() {
    let mut s = session();
    s.apply(InputEvent::Command(Command::SetSignedMode(
        SignedMode::OnesComplement,
    )))
    .unwrap();
    enter_literal(&mut s, "-0").unwrap();
    assert_eq!(
        s.snapshot().calculator.stack[0]
            .as_integer()
            .unwrap()
            .bits(),
        255
    );
    s.apply(InputEvent::Command(Command::SetNumericMode(
        NumericMode::FloatingPoint,
    )))
    .unwrap();
    for (text, bits) in [("-0", (-0.0_f64).to_bits()), ("-", 0)] {
        enter_literal(&mut s, text).unwrap();
        assert_eq!(
            s.snapshot().calculator.stack[0]
                .as_float()
                .unwrap()
                .to_bits(),
            bits
        );
    }
    let before = s.snapshot();
    assert_eq!(
        enter_literal(&mut s, "1e-"),
        Err(LiteralEntryError::Literal(LiteralError::InvalidNumber))
    );
    assert_eq!(s.snapshot(), before);
}
