use Command::*;
use InputEvent::{Command as C, Key as K};
use Key::*;
use rpn_core::*;
use rpn_input::*;
fn session(float: bool) -> HpInputSession {
    let mut s = HpInputSession::new(Calculator::new());
    if float {
        s.apply(C(SetNumericMode(NumericMode::FloatingPoint)))
            .unwrap()
    }
    s
}
fn rejected(s: &mut HpInputSession, event: InputEvent, why: SessionError) {
    let before = s.snapshot();
    assert_eq!(s.apply(event), Err(why));
    assert_eq!(s.snapshot(), before);
}
#[test]
fn incomplete_exponent_priorities() {
    let mut s = session(true);
    rejected(&mut s, K(Exponent), InputError::IncompleteEntry.into());
    for k in [Digit('1'), Exponent, Key::ChangeSign] {
        s.apply(K(k)).unwrap();
    }
    for cmd in [
        Divide,
        Enter,
        RollUp,
        SwapXY,
        Store(0),
        Recall(0),
        SetRadix(Radix::Decimal),
    ] {
        rejected(&mut s, C(cmd), InputError::IncompleteEntry.into());
    }
    for cmd in [
        SetRadix(Radix::Hexadecimal),
        SetWordSize(WordSize::new(8).unwrap()),
        SetSignedMode(SignedMode::Unsigned),
    ] {
        rejected(&mut s, C(cmd), CalcError::NumericModeMismatch.into());
    }
    rejected(&mut s, C(Store(16)), CalcError::InvalidRegister(16).into());
    s.apply(C(ClearRegisters)).unwrap();
    assert_eq!(s.snapshot().input_buffer.as_deref(), Some("1e-"));
    s.apply(C(ClearX)).unwrap();
    assert_eq!(s.snapshot().entry_phase, EntryPhase::Idle);
}
#[test]
fn exponent_backspace_and_sign() {
    let mut s = session(true);
    for k in [Digit('1'), Exponent, Digit('2'), Key::ChangeSign] {
        s.apply(K(k)).unwrap();
    }
    assert_eq!(s.snapshot().calculator.stack[0], Number::Float(0.01));
    for (buf, part) in [
        ("1e-", EntryPart::Exponent),
        ("1e", EntryPart::Exponent),
        ("1", EntryPart::Mantissa),
    ] {
        s.apply(K(Backspace)).unwrap();
        assert_eq!(s.snapshot().input_buffer.as_deref(), Some(buf));
        assert_eq!(s.snapshot().entry_part, part);
    }
    s.apply(K(Backspace)).unwrap();
    s.apply(C(Command::ChangeSign)).unwrap();
    assert_eq!(
        s.snapshot().calculator.stack[0]
            .as_float()
            .unwrap()
            .to_bits(),
        (-0.0f64).to_bits()
    );
}
#[test]
fn all_widths_and_radices() {
    for bits in 1..=64 {
        let size = WordSize::new(bits).unwrap();
        for radix in [
            Radix::Binary,
            Radix::Octal,
            Radix::Decimal,
            Radix::Hexadecimal,
        ] {
            for signed_mode in [
                SignedMode::Unsigned,
                SignedMode::OnesComplement,
                SignedMode::TwosComplement,
            ] {
                let mut s = HpInputSession::new(
                    Calculator::with_config(CalculatorConfig {
                        word_config: WordConfig {
                            ones_zero_mode: rpn_core::OnesZeroMode::Canonical,
                            size,
                            signed_mode,
                        },
                        radix,
                        ..Default::default()
                    })
                    .unwrap(),
                );
                let max = if radix == Radix::Decimal && signed_mode != SignedMode::Unsigned {
                    size.mask() >> 1
                } else {
                    size.mask()
                };
                let digits = match radix {
                    Radix::Binary => format!("{max:b}"),
                    Radix::Octal => format!("{max:o}"),
                    Radix::Decimal => max.to_string(),
                    Radix::Hexadecimal => format!("{max:x}"),
                };
                for c in digits.chars() {
                    s.apply(K(Digit(c))).unwrap();
                }
                assert_eq!(
                    s.snapshot().calculator.stack[0]
                        .as_integer()
                        .unwrap()
                        .bits(),
                    max
                );
                rejected(&mut s, K(Digit('1')), CalcError::IntegerOutOfRange.into());
            }
        }
    }
}
#[test]
fn length_and_invalid_digits() {
    let mut s = session(false);
    for c in ['-', '\u{ff11}', 'G'] {
        rejected(
            &mut s,
            K(Digit(c)),
            InputError::InvalidDigit {
                digit: c,
                radix: Radix::Hexadecimal,
            }
            .into(),
        );
    }
    for _ in 0..256 {
        s.apply(K(Digit('0'))).unwrap();
    }
    rejected(&mut s, K(Digit('G')), InputError::InputTooLong.into());
    s.apply(K(Backspace)).unwrap();
    assert_eq!(s.snapshot().input_buffer.unwrap().len(), 255);
}
#[test]
fn lift_registers_and_reset() {
    let mut s = session(false);
    let initial = s.snapshot();
    for event in [K(Digit('a')), C(Store(0)), K(Digit('2')), C(Recall(0))] {
        s.apply(event).unwrap();
    }
    let snap = s.snapshot().calculator;
    assert_eq!(
        snap.stack[..3]
            .iter()
            .map(|n| n.as_integer().unwrap().bits())
            .collect::<Vec<_>>(),
        [10, 2, 10]
    );
    s.apply(K(Digit('3'))).unwrap();
    assert_eq!(s.test_bit(0), Err(InputError::InputInProgress.into()));
    let n = s.snapshot().calculator.stack[0];
    rejected(&mut s, C(Push(n)), InputError::InputInProgress.into());
    s.apply(C(ClearRegisters)).unwrap();
    assert_eq!(s.snapshot().input_buffer.as_deref(), Some("3"));
    s.reset();
    assert_eq!(s.snapshot(), initial);
}
#[test]
fn point_and_mode_conversion_rollback() {
    let mut s = session(true);
    s.apply(K(DecimalPoint)).unwrap();
    s.apply(K(Digit('5'))).unwrap();
    rejected(&mut s, K(DecimalPoint), InputError::InvalidNumber.into());
    rejected(
        &mut s,
        C(SetNumericMode(NumericMode::Integer)),
        CalcError::IntegerOutOfRange.into(),
    );
    s.apply(C(SetRadix(Radix::Decimal))).unwrap();
    assert_eq!(s.snapshot().calculator.stack[0], Number::Float(0.5));
}
