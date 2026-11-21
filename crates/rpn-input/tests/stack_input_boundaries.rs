//! Acceptance fixtures SI-T027 through SI-T076 (stack/input test specification).
use rpn_core::{Command::*, NumericMode::*, Radix::*, SignedMode::*, StackLift::*, *};
use rpn_input::*;

#[derive(Clone)]
struct Fixture {
    config: CalculatorConfig,
    stack: [Number; 4],
    last_x: Number,
    r0: Number,
    lift: StackLift,
    buffer: Option<String>,
}

fn word(bits: u64, width: u8) -> Number {
    Number::Integer(Word::from_bits(bits, WordSize::new(width).unwrap()))
}

impl Fixture {
    fn integer(stack: [u64; 4], width: u8, lift: StackLift) -> Self {
        let mut config = CalculatorConfig {
            radix: Decimal,
            ..Default::default()
        };
        config.word_config.size = WordSize::new(width).unwrap();
        Self {
            config,
            stack: stack.map(|x| word(x, width)),
            last_x: word(9, width),
            r0: word(0, width),
            lift,
            buffer: None,
        }
    }
    fn float(stack: [f64; 4], lift: StackLift) -> Self {
        Self {
            config: CalculatorConfig {
                radix: Decimal,
                numeric_mode: FloatingPoint,
                ..Default::default()
            },
            stack: stack.map(Number::Float),
            last_x: Number::Float(9.0),
            r0: Number::Float(0.0),
            lift,
            buffer: None,
        }
    }
    fn editing(mut self, buffer: impl Into<String>) -> Self {
        self.buffer = Some(buffer.into());
        self
    }
    fn radix(mut self, radix: Radix) -> Self {
        self.config.radix = radix;
        self
    }
    fn unsigned(mut self) -> Self {
        self.config.word_config.signed_mode = Unsigned;
        self
    }
    fn history(mut self, n: Number) -> Self {
        self.last_x = n;
        self
    }
    fn register(mut self, n: Number) -> Self {
        self.r0 = n;
        self
    }

    fn core(&self, creation: CalculatorConfig, flags: [bool; 6]) -> Calculator {
        let mut core = Calculator::with_config(creation).unwrap();
        if creation != self.config {
            core.execute_all([
                SetRadix(self.config.radix),
                SetSignedMode(self.config.word_config.signed_mode),
            ])
            .unwrap();
        }
        core.execute_all([Push(self.last_x), Absolute, Push(self.r0), Store(0)])
            .unwrap();
        for (i, value) in flags.into_iter().enumerate() {
            core.execute(SetFlag {
                index: i as u8,
                value,
            })
            .unwrap();
        }
        let [x, y, z, t] = self.stack;
        core.execute_all([Push(t), Push(z), Push(y)]).unwrap();
        if self.lift == Disabled {
            core.execute(Enter).unwrap();
        }
        if self.buffer.is_none() {
            core.update_value(ValueUpdate::Start(x)).unwrap();
        }
        core
    }

    fn session(&self, creation: CalculatorConfig, flags: [bool; 6]) -> HpInputSession {
        let mut s = HpInputSession::new(self.core(creation, flags));
        if let Some(buffer) = &self.buffer {
            // A leading minus is entered by changing the mantissa sign after its digits.
            let digits = buffer.strip_prefix('-').unwrap_or(buffer);
            for ch in digits.chars() {
                key(
                    &mut s,
                    match ch {
                        '.' => Key::DecimalPoint,
                        'e' => Key::Exponent,
                        '-' => Key::ChangeSign,
                        _ => Key::Digit(ch),
                    },
                );
            }
            if buffer.starts_with('-') {
                key(&mut s, Key::ChangeSign);
            }
        }
        self.check(&s, flags);
        s
    }

    #[track_caller]
    fn check(&self, s: &HpInputSession, flags: [bool; 6]) {
        let snap = s.snapshot();
        let core = snap.calculator;
        assert_eq!(core.numeric_mode, self.config.numeric_mode);
        assert_eq!(core.word_config, self.config.word_config);
        assert_eq!(core.radix, self.config.radix);
        assert_eq!(core.stack_lift, self.lift);
        for (actual, expected) in core.stack.into_iter().zip(self.stack) {
            check_number(actual, expected);
        }
        check_number(core.last_x, self.last_x);
        let zero = match self.config.numeric_mode {
            Integer => word(0, self.config.word_config.size.bits()),
            FloatingPoint => Number::Float(0.0),
        };
        assert_eq!(core.registers.len(), self.config.register_count);
        for (i, actual) in core.registers.into_iter().enumerate() {
            check_number(actual, if i == 0 { self.r0 } else { zero });
        }
        assert_eq!(
            [0, 1, 2, 3, 4, 5].map(|i| core.flags.get(i).unwrap()),
            flags
        );
        assert_eq!(snap.input_buffer, self.buffer);
        assert_eq!(
            snap.entry_phase,
            if self.buffer.is_some() {
                EntryPhase::Editing
            } else {
                EntryPhase::Idle
            }
        );
        assert_eq!(
            snap.entry_part,
            if self.buffer.as_deref().is_some_and(|b| b.contains('e')) {
                EntryPart::Exponent
            } else {
                EntryPart::Mantissa
            }
        );
    }
}

#[track_caller]
fn check_number(actual: Number, expected: Number) {
    match (actual, expected) {
        (Number::Float(a), Number::Float(b)) => assert_eq!(a.to_bits(), b.to_bits()),
        _ => assert_eq!(actual, expected),
    }
}

fn flag_sets() -> impl Iterator<Item = [bool; 6]> {
    [(false, false), (false, true), (true, false), (true, true)]
        .map(|(c, g)| [true, false, true, false, c, g])
        .into_iter()
}
#[track_caller]
fn verify(initial: Fixture, expected: Fixture, action: impl Fn(&mut HpInputSession)) {
    for flags in flag_sets() {
        let mut s = initial.session(initial.config, flags);
        action(&mut s);
        expected.check(&s, flags);
    }
}
#[track_caller]
fn key(s: &mut HpInputSession, k: Key) {
    s.apply(InputEvent::Key(k)).unwrap();
}
#[track_caller]
fn command(s: &mut HpInputSession, c: Command) {
    s.apply(InputEvent::Command(c)).unwrap();
}
#[track_caller]
fn reject(s: &mut HpInputSession, event: InputEvent, error: impl Into<SessionError>) {
    let before = s.snapshot();
    assert_eq!(s.apply(event), Err(error.into()));
    assert_eq!(s.snapshot(), before);
}

#[test]
fn si_t027_float_chs_then_digit() {
    let initial = Fixture::float([5.0, 2.0, 3.0, 4.0], Disabled).editing("5");
    let expected = Fixture::float([-52.0, 2.0, 3.0, 4.0], Disabled).editing("-52");
    verify(initial, expected, |s| {
        key(s, Key::ChangeSign);
        key(s, Key::Digit('2'));
    });
}

#[test]
fn si_t028_float_double_chs() {
    let initial = Fixture::float([5.0, 2.0, 3.0, 4.0], Enabled).editing("5");
    let expected = initial.clone();
    verify(initial, expected, |s| {
        key(s, Key::ChangeSign);
        key(s, Key::ChangeSign);
    });
}

#[test]
fn si_t029_idle_float_chs() {
    let initial = Fixture::float([5.0, 2.0, 3.0, 4.0], Disabled);
    let expected = Fixture::float([-5.0, 2.0, 3.0, 4.0], Enabled);
    verify(initial, expected, |s| {
        key(s, Key::ChangeSign);
    });
}

#[test]
fn si_t030_float_negative_zero() {
    let initial = Fixture::float([0.0, 2.0, 3.0, 4.0], Disabled).editing("0");
    let expected = Fixture::float([-0.0, 2.0, 3.0, 4.0], Disabled).editing("-0");
    verify(initial, expected, |s| {
        key(s, Key::ChangeSign);
    });
}

#[test]
fn si_t031_start_decimal_point() {
    let initial = Fixture::float([8.0, 2.0, 3.0, 4.0], Enabled);
    let expected = Fixture::float([0.5, 8.0, 2.0, 3.0], Enabled).editing("0.5");
    verify(initial, expected, |s| {
        key(s, Key::DecimalPoint);
        key(s, Key::Digit('5'));
    });
}

#[test]
fn si_t032_duplicate_point() {
    let initial = Fixture::float([1.2, 2.0, 3.0, 4.0], Disabled).editing("1.2");
    let expected = initial.clone();
    verify(initial, expected, |s| {
        reject(
            s,
            InputEvent::Key(Key::DecimalPoint),
            InputError::InvalidNumber,
        );
    });
}

#[test]
fn si_t033_integer_point() {
    let initial = Fixture::integer([1, 2, 3, 4], 16, Disabled).editing("1");
    let expected = initial.clone();
    verify(initial, expected, |s| {
        reject(
            s,
            InputEvent::Key(Key::DecimalPoint),
            CalcError::NumericModeMismatch,
        );
    });
}

#[test]
fn si_t034_exponent_sign() {
    let initial = Fixture::float([1.0, 2.0, 3.0, 4.0], Disabled).editing("1");
    let expected = Fixture::float([0.01, 2.0, 3.0, 4.0], Disabled).editing("1e-2");
    verify(initial, expected, |s| {
        key(s, Key::Exponent);
        key(s, Key::Digit('2'));
        key(s, Key::ChangeSign);
    });
}

#[test]
fn si_t035_delete_exponent() {
    let initial = Fixture::float([0.01, 2.0, 3.0, 4.0], Enabled).editing("1e-2");
    let expected = Fixture::float([1.0, 2.0, 3.0, 4.0], Enabled).editing("1");
    verify(initial, expected, |s| {
        key(s, Key::Backspace);
        key(s, Key::Backspace);
        key(s, Key::Backspace);
    });
}

#[test]
fn si_t036_delete_last_digit() {
    let initial = Fixture::integer([1, 2, 3, 4], 16, Enabled).editing("1");
    let expected = Fixture::integer([2, 2, 3, 4], 16, Enabled).editing("02");
    verify(initial, expected, |s| {
        key(s, Key::Backspace);
        key(s, Key::Digit('2'));
    });
}

#[test]
fn si_t037_idle_backspace() {
    let initial = Fixture::integer([5, 2, 3, 4], 16, Enabled);
    let expected = Fixture::integer([0, 2, 3, 4], 16, Disabled);
    verify(initial, expected, |s| {
        key(s, Key::Backspace);
    });
}

#[test]
fn si_t038_idle_exponent() {
    let initial = Fixture::float([5.0, 2.0, 3.0, 4.0], Enabled);
    let expected = initial.clone();
    verify(initial, expected, |s| {
        reject(
            s,
            InputEvent::Key(Key::Exponent),
            InputError::IncompleteEntry,
        );
    });
}

#[test]
fn si_t039_incomplete_enter() {
    let initial = Fixture::float([1.0, 2.0, 3.0, 4.0], Enabled).editing("1e");
    let expected = initial.clone();
    verify(initial, expected, |s| {
        reject(s, InputEvent::Command(Enter), InputError::IncompleteEntry);
    });
}

#[test]
fn si_t040_incomplete_commands() {
    let initial = Fixture::float([1.0, 2.0, 3.0, 4.0], Disabled).editing("1e-");
    let expected = initial.clone();
    verify(initial, expected, |s| {
        for c in [
            RollDown,
            RollUp,
            SwapXY,
            Add,
            Store(0),
            Recall(0),
            RecallLastX,
            SetRadix(Decimal),
            SetNumericMode(FloatingPoint),
        ] {
            reject(s, InputEvent::Command(c), InputError::IncompleteEntry);
        }
    });
}

#[test]
fn si_t041_change_base_disabled() {
    let initial = Fixture::integer([10, 2, 3, 4], 16, Disabled)
        .editing("A")
        .radix(Hexadecimal);
    let expected = Fixture::integer([2, 2, 3, 4], 16, Disabled).editing("2");
    verify(initial, expected, |s| {
        command(s, SetRadix(Decimal));
        key(s, Key::Digit('2'));
    });
}

#[test]
fn si_t042_change_base_enabled() {
    let initial = Fixture::integer([10, 2, 3, 4], 16, Enabled)
        .editing("A")
        .radix(Hexadecimal);
    let expected = Fixture::integer([2, 10, 2, 3], 16, Enabled).editing("2");
    verify(initial, expected, |s| {
        command(s, SetRadix(Decimal));
        key(s, Key::Digit('2'));
    });
}

#[test]
fn si_t043_same_radix() {
    let initial = Fixture::integer([12, 2, 3, 4], 16, Disabled).editing("12");
    let expected = Fixture::integer([3, 2, 3, 4], 16, Disabled).editing("3");
    verify(initial, expected, |s| {
        command(s, SetRadix(Decimal));
        key(s, Key::Digit('3'));
    });
}

#[test]
fn si_t044_radix_round_trip() {
    let initial = Fixture::integer([255, 2, 3, 4], 16, Enabled).radix(Hexadecimal);
    let expected = initial.clone();
    verify(initial, expected, |s| {
        for radix in [Binary, Octal, Decimal, Hexadecimal] {
            command(s, SetRadix(radix));
            assert_eq!(s.snapshot().calculator.radix, radix);
        }
    });
}

#[test]
fn si_t045_invalid_first_digit() {
    let initial = Fixture::integer([5, 2, 3, 4], 16, Enabled).radix(Binary);
    let expected = initial.clone();
    verify(initial, expected, |s| {
        reject(
            s,
            InputEvent::Key(Key::Digit('2')),
            InputError::InvalidDigit {
                digit: '2',
                radix: Binary,
            },
        );
    });
}

#[test]
fn si_t046_invalid_edit_digit() {
    let initial = Fixture::integer([7, 2, 3, 4], 16, Disabled)
        .editing("7")
        .radix(Octal);
    let expected = initial.clone();
    verify(initial, expected, |s| {
        reject(
            s,
            InputEvent::Key(Key::Digit('8')),
            InputError::InvalidDigit {
                digit: '8',
                radix: Octal,
            },
        );
    });
}

#[test]
fn si_t047_lowercase_hex() {
    let initial = Fixture::integer([5, 2, 3, 4], 16, Disabled).radix(Hexadecimal);
    let expected = Fixture::integer([175, 2, 3, 4], 16, Disabled)
        .editing("AF")
        .radix(Hexadecimal);
    verify(initial, expected, |s| {
        key(s, Key::Digit('a'));
        key(s, Key::Digit('f'));
    });
}

#[test]
fn si_t048_maximum_raw() {
    let initial = Fixture::integer([15, 2, 3, 4], 8, Disabled)
        .editing("F")
        .radix(Hexadecimal);
    let expected = Fixture::integer([255, 2, 3, 4], 8, Disabled)
        .editing("FF")
        .radix(Hexadecimal);
    verify(initial, expected, |s| {
        key(s, Key::Digit('F'));
        let core = s.snapshot().calculator;
        assert_eq!(
            core.stack[0]
                .as_integer()
                .unwrap()
                .to_signed(&core.word_config),
            -1
        );
    });
}

#[test]
fn si_t049_overflowing_digit() {
    let initial = Fixture::integer([255, 2, 3, 4], 8, Enabled)
        .editing("FF")
        .radix(Hexadecimal);
    let expected = initial.clone();
    verify(initial, expected, |s| {
        reject(
            s,
            InputEvent::Key(Key::Digit('F')),
            CalcError::IntegerOutOfRange,
        );
    });
}

#[test]
fn si_t050_one_bit_first_digit() {
    let initial = Fixture::integer([1, 0, 0, 0], 1, Enabled)
        .radix(Hexadecimal)
        .unsigned()
        .history(word(0, 1));
    let expected = initial.clone();
    verify(initial, expected, |s| {
        reject(
            s,
            InputEvent::Key(Key::Digit('2')),
            CalcError::IntegerOutOfRange,
        );
    });
}

#[test]
fn si_t051_accept_256th_digit() {
    let initial = Fixture::integer([0, 2, 3, 4], 16, Disabled).editing("0".repeat(255));
    let expected = Fixture::integer([0, 2, 3, 4], 16, Disabled).editing("0".repeat(256));
    verify(initial, expected, |s| {
        key(s, Key::Digit('0'));
    });
}

#[test]
fn si_t052_reject_257th_digit() {
    let initial = Fixture::integer([0, 2, 3, 4], 16, Enabled).editing("0".repeat(256));
    let expected = initial.clone();
    verify(initial, expected, |s| {
        reject(
            s,
            InputEvent::Key(Key::Digit('0')),
            InputError::InputTooLong,
        );
    });
}

#[test]
fn si_t053_resize_all_words() {
    let initial = Fixture::integer([0x1234, 0xff, 3, 4], 16, Enabled)
        .editing("1234")
        .radix(Hexadecimal)
        .history(word(0x1234, 16))
        .register(word(0x5678, 16));
    let expected = Fixture::integer([0x34, 0xff, 3, 4], 16, Enabled)
        .radix(Hexadecimal)
        .history(word(0x34, 16))
        .register(word(0x78, 16));
    verify(initial, expected, |s| {
        command(s, SetWordSize(WordSize::new(8).unwrap()));
        command(s, SetWordSize(WordSize::new(16).unwrap()));
    });
}

#[test]
fn si_t054_reinterpret_unsigned() {
    let initial = Fixture::integer([255, 2, 3, 4], 8, Disabled)
        .editing("FF")
        .radix(Hexadecimal);
    let expected = Fixture::integer([255, 2, 3, 4], 8, Disabled)
        .radix(Hexadecimal)
        .unsigned();
    verify(initial, expected, |s| {
        command(s, SetSignedMode(Unsigned));
        let core = s.snapshot().calculator;
        assert_eq!(
            core.stack[0]
                .as_integer()
                .unwrap()
                .to_signed(&core.word_config),
            255
        );
    });
}

#[test]
fn si_t055_convert_to_float() {
    let initial = Fixture::integer([5, 2, 3, 4], 16, Disabled)
        .editing("5")
        .radix(Hexadecimal)
        .register(word(7, 16));
    let expected = Fixture::float([5.0, 2.0, 3.0, 4.0], Disabled).register(Number::Float(7.0));
    verify(initial, expected, |s| {
        command(s, SetNumericMode(FloatingPoint));
    });
}

#[test]
fn si_t056_inexact_register_blocks_conversion() {
    let initial = Fixture::integer([5, 2, 3, 4], 64, Enabled)
        .editing("5")
        .unsigned()
        .register(word(9_007_199_254_740_993, 64));
    let expected = initial.clone();
    verify(initial, expected, |s| {
        reject(
            s,
            InputEvent::Command(SetNumericMode(FloatingPoint)),
            CalcError::IntegerOutOfRange,
        );
    });
}

#[test]
fn si_t057_fraction_blocks_conversion() {
    let initial = Fixture::float([1.5, 2.0, 3.0, 4.0], Disabled).editing("1.5");
    let expected = initial.clone();
    verify(initial, expected, |s| {
        reject(
            s,
            InputEvent::Command(SetNumericMode(Integer)),
            CalcError::IntegerOutOfRange,
        );
    });
}

#[test]
fn si_t058_reject_hex_in_float() {
    let initial = Fixture::float([5.0, 2.0, 3.0, 4.0], Enabled).editing("5");
    let expected = initial.clone();
    verify(initial, expected, |s| {
        reject(
            s,
            InputEvent::Command(SetRadix(Hexadecimal)),
            CalcError::NumericModeMismatch,
        );
    });
}

#[test]
fn si_t059_decimal_in_float() {
    let initial = Fixture::float([5.0, 2.0, 3.0, 4.0], Disabled).editing("5");
    let expected = Fixture::float([5.0, 2.0, 3.0, 4.0], Disabled);
    verify(initial, expected, |s| {
        command(s, SetRadix(Decimal));
    });
}

#[test]
fn si_t060_same_numeric_mode() {
    let initial = Fixture::integer([10, 2, 3, 4], 16, Enabled)
        .editing("A")
        .radix(Hexadecimal);
    let expected = Fixture::integer([10, 2, 3, 4], 16, Enabled).radix(Hexadecimal);
    verify(initial, expected, |s| {
        command(s, SetNumericMode(Integer));
    });
}

#[test]
fn si_t061_zero_divisor() {
    let initial = Fixture::integer([0, 5, 3, 4], 16, Enabled).editing("0");
    let expected = initial.clone();
    verify(initial, expected, |s| {
        reject(s, InputEvent::Command(Divide), CalcError::DivisionByZero);
    });
}

#[test]
fn si_t062_resume_after_division_error() {
    let initial = Fixture::integer([0, 5, 3, 4], 16, Enabled).editing("0");
    let expected = Fixture::integer([2, 5, 3, 4], 16, Enabled).editing("02");
    verify(initial, expected, |s| {
        reject(s, InputEvent::Command(Divide), CalcError::DivisionByZero);
        key(s, Key::Digit('2'));
    });
}

#[test]
fn si_t063_clear_after_division_error() {
    let initial = Fixture::integer([0, 5, 3, 4], 16, Enabled).editing("0");
    let expected = Fixture::integer([2, 5, 3, 4], 16, Disabled).editing("2");
    verify(initial, expected, |s| {
        reject(s, InputEvent::Command(Divide), CalcError::DivisionByZero);
        command(s, ClearX);
        key(s, Key::Digit('2'));
    });
}

#[test]
fn si_t064_negative_square_root() {
    let initial = Fixture::float([-1.0, 2.0, 3.0, 4.0], Disabled).editing("-1");
    let expected = initial.clone();
    verify(initial, expected, |s| {
        reject(s, InputEvent::Command(Sqrt), CalcError::DomainError);
    });
}

#[test]
fn si_t065_invalid_register() {
    let initial = Fixture::integer([12, 2, 3, 4], 16, Disabled).editing("12");
    let expected = initial.clone();
    verify(initial, expected, |s| {
        for c in [Store(16), Recall(16)] {
            reject(s, InputEvent::Command(c), CalcError::InvalidRegister(16));
        }
    });
}

#[test]
fn si_t066_clear_stack() {
    let initial = Fixture::integer([1, 2, 3, 4], 16, Enabled)
        .editing("1")
        .register(word(7, 16));
    let expected = Fixture::integer([0, 0, 0, 0], 16, Disabled)
        .history(word(0, 16))
        .register(word(7, 16));
    verify(initial, expected, |s| {
        command(s, ClearStack);
    });
}

#[test]
fn si_t067_clear_registers() {
    let initial = Fixture::integer([1, 2, 3, 4], 16, Disabled)
        .editing("1")
        .register(word(7, 16));
    let expected = Fixture::integer([1, 2, 3, 4], 16, Disabled).editing("1");
    verify(initial, expected, |s| {
        command(s, ClearRegisters);
    });
}

#[test]
fn si_t068_signed_add_overflow() {
    let initial = Fixture::integer([1, 127, 3, 4], 8, Disabled);
    for flags in flag_sets() {
        let mut s = initial.session(initial.config, flags);
        command(&mut s, Add);
        let mut result = flags;
        result[4] = false;
        result[5] = true;
        Fixture::integer([128, 3, 4, 4], 8, Enabled)
            .history(word(1, 8))
            .check(&s, result);
    }
}

#[test]
fn si_t069_remainder_keeps_flags() {
    let initial = Fixture::integer([4, 7, 3, 4], 16, Disabled);
    let expected = Fixture::integer([3, 3, 4, 4], 16, Enabled).history(word(4, 16));
    verify(initial, expected, |s| {
        command(s, Remainder);
    });
}

#[test]
fn si_t070_execute_all_stops() {
    let initial = Fixture::integer([1, 2, 3, 4], 16, Disabled);
    for flags in flag_sets() {
        let mut core = initial.core(initial.config, flags);
        initial.check(&HpInputSession::new(core.clone()), flags);
        assert_eq!(
            core.execute_all([
                Push(word(5, 16)),
                Push(word(0, 16)),
                Divide,
                Push(word(7, 16))
            ]),
            Err(CalcError::DivisionByZero)
        );
        Fixture::integer([0, 5, 2, 3], 16, Enabled).check(&HpInputSession::new(core), flags);
    }
}

#[test]
fn si_t071_independent_snapshot() {
    let initial = Fixture::integer([12, 2, 3, 4], 16, Enabled)
        .editing("12")
        .register(word(7, 16));
    let expected = initial.clone();
    verify(initial, expected, |s| {
        let mut copy = s.snapshot();
        copy.calculator.stack[0] = word(99, 16);
        copy.calculator.flags = Flags::default();
        copy.input_buffer.as_mut().unwrap().clear();
        copy.calculator.registers[0] = word(99, 16);
        assert_ne!(copy, s.snapshot());
    });
}

#[test]
fn si_t072_restore_creation_config() {
    let initial = Fixture::integer([1, 2, 3, 4], 8, Enabled);
    let expected = Fixture::integer([0, 0, 0, 0], 8, Disabled)
        .radix(Hexadecimal)
        .unsigned()
        .history(word(0, 8));
    let mut s = initial.session(expected.config, [true, false, true, false, true, true]);
    s.reset();
    expected.check(&s, [false; 6]);
}

#[test]
fn si_t073_edit_large_exponent() {
    let initial = Fixture::float([1e306, 2.0, 3.0, 4.0], Disabled).editing("1e306");
    let expected = Fixture::float([1e-30, 2.0, 3.0, 4.0], Disabled).editing("1e-30");
    verify(initial, expected, |s| {
        key(s, Key::ChangeSign);
        key(s, Key::Backspace);
    });
}

#[test]
fn si_t074_invalid_ascii_and_unicode_digits() {
    let initial = Fixture::integer([1, 2, 3, 4], 16, Enabled);
    let expected = initial.clone();
    verify(initial, expected, |s| {
        for digit in ['-', '\u{ff11}', 'G'] {
            reject(
                s,
                InputEvent::Key(Key::Digit(digit)),
                InputError::InvalidDigit {
                    digit,
                    radix: Decimal,
                },
            );
        }
    });
}

#[test]
fn si_t075_reject_word_settings_in_float() {
    let initial = Fixture::float([1.0, 2.0, 3.0, 4.0], Disabled).editing("1");
    let expected = initial.clone();
    verify(initial, expected, |s| {
        for c in [
            SetWordSize(WordSize::new(8).unwrap()),
            SetSignedMode(Unsigned),
        ] {
            reject(s, InputEvent::Command(c), CalcError::NumericModeMismatch);
        }
    });
}

#[test]
fn si_t076_negate_minimum_integer() {
    let initial = Fixture::integer([128, 2, 3, 4], 8, Disabled)
        .editing("80")
        .radix(Hexadecimal);
    for flags in flag_sets() {
        let mut s = initial.session(initial.config, flags);
        key(&mut s, Key::ChangeSign);
        let mut result = flags;
        // CHS preserves Carry. SI-T076 starts with Carry set.
        result[5] = true;
        Fixture::integer([128, 2, 3, 4], 8, Enabled)
            .radix(Hexadecimal)
            .history(word(128, 8))
            .check(&s, result);
    }
}
