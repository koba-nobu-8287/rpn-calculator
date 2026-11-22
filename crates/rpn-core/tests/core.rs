use rpn_core::*;

fn size(bits: u8) -> WordSize {
    WordSize::new(bits).unwrap()
}

fn int(raw: u64, bits: u8) -> Number {
    Number::Integer(Word::from_bits(raw, size(bits)))
}

fn config(bits: u8, signed_mode: SignedMode) -> CalculatorConfig {
    CalculatorConfig {
        word_config: WordConfig {
            ones_zero_mode: rpn_core::OnesZeroMode::Canonical,
            size: size(bits),
            signed_mode,
        },
        ..CalculatorConfig::default()
    }
}

fn raw(calc: &Calculator) -> u64 {
    calc.snapshot().stack[0].as_integer().unwrap().bits()
}

#[test]
fn test_067_float_rounding_subnormal_and_overflow_preserve_flags() {
    use Command::*;
    for carry in [false, true] {
        for overflow in [false, true] {
            let mut c = Calculator::new();
            c.execute_all([
                SetNumericMode(NumericMode::FloatingPoint),
                SetFlag {
                    index: 4,
                    value: carry,
                },
                SetFlag {
                    index: 5,
                    value: overflow,
                },
            ])
            .unwrap();
            let flags = c.snapshot().flags;
            for (y, x, op, expected) in [
                (0.1, 0.2, Add, 0.30000000000000004_f64),
                (f64::MIN_POSITIVE, 2.0, Divide, f64::from_bits(1_u64 << 51)),
            ] {
                c.execute_all([Push(Number::Float(y)), Push(Number::Float(x)), op])
                    .unwrap();
                assert_eq!(
                    c.snapshot().stack[0].as_float().unwrap().to_bits(),
                    expected.to_bits()
                );
                assert_eq!(c.snapshot().flags, flags);
            }
            c.execute_all([Push(Number::Float(f64::MAX)), Push(Number::Float(2.0))])
                .unwrap();
            expect_error(&mut c, Multiply, CalcError::FloatingPointOutOfRange);
        }
    }
}

#[test]
fn test_041_sign_transitions_preserve_raw_state_at_both_lift_settings() {
    use Command::*;
    use SignedMode::*;
    let modes = [Unsigned, OnesComplement, TwosComplement];
    for source in modes {
        for target in modes.into_iter().filter(|&target| target != source) {
            for lift in [StackLift::Enabled, StackLift::Disabled] {
                for (carry, overflow) in
                    [(false, false), (false, true), (true, false), (true, true)]
                {
                    let mut c = Calculator::with_config(config(8, source)).unwrap();
                    c.execute_all([
                        Push(int(255, 8)),
                        Absolute,
                        Push(int(255, 8)),
                        Store(0),
                        SetFlag {
                            index: 4,
                            value: carry,
                        },
                        SetFlag {
                            index: 5,
                            value: overflow,
                        },
                    ])
                    .unwrap();
                    if lift == StackLift::Disabled {
                        c.execute(Enter).unwrap();
                    }
                    let mut expected = c.snapshot();
                    expected.word_config.signed_mode = target;
                    c.execute(SetSignedMode(target)).unwrap();
                    assert_eq!(c.snapshot(), expected);
                    let x = c.snapshot().stack[0].as_integer().unwrap();
                    let value = match target {
                        Unsigned => 255,
                        OnesComplement => 0,
                        TwosComplement => -1,
                    };
                    assert_eq!(x.to_signed(&expected.word_config), value);
                    assert_eq!(
                        x.is_negative_zero(&expected.word_config),
                        target == OnesComplement
                    );
                }
            }
        }
    }
}

fn expect_error(calc: &mut Calculator, command: Command, error: CalcError) {
    let before = calc.snapshot();
    assert_eq!(calc.execute(command), Err(error), "{command:?}");
    let after = calc.snapshot();
    assert_eq!(after, before);
    for (a, b) in after
        .stack
        .iter()
        .chain([&after.last_x])
        .chain(&after.registers)
        .zip(
            before
                .stack
                .iter()
                .chain([&before.last_x])
                .chain(&before.registers),
        )
    {
        assert_eq!(
            a.as_float().map(f64::to_bits),
            b.as_float().map(f64::to_bits)
        );
    }
}

// TEST-001–009 / IA-C001,006,007.
#[test]
fn word_widths_and_signed_interpretations() {
    assert_eq!(WordSize::new(0), Err(CalcError::InvalidWordSize(0)));
    for bits in 65..=255 {
        assert_eq!(WordSize::new(bits), Err(CalcError::InvalidWordSize(bits)));
    }
    for bits in 1..=64 {
        let width = size(bits);
        let mask = (1_u128 << bits) - 1;
        assert_eq!(width.bits(), bits);
        assert_eq!(u128::from(width.mask()), mask);
        let word = Word::from_bits(u64::MAX, width);
        for (mode, value, negative) in [
            (SignedMode::Unsigned, mask as i128, false),
            (SignedMode::TwosComplement, -1, true),
            (SignedMode::OnesComplement, 0, true),
        ] {
            let cfg = config(bits, mode).word_config;
            assert_eq!(word.to_signed(&cfg), value);
            assert_eq!(word.is_negative(&cfg), negative);
            assert_eq!(
                word.is_negative_zero(&cfg),
                mode == SignedMode::OnesComplement
            );
            assert_eq!(word.bits(), mask as u64);
        }
    }
}

#[test]
fn default_reset_config_and_snapshot_ownership() {
    let default = Calculator::new().snapshot();
    assert_eq!(default.stack, [int(0, 16); 4]);
    assert_eq!(default.last_x, int(0, 16));
    assert_eq!(default.registers, vec![int(0, 16); 16]);
    assert_eq!(default.stack_lift, StackLift::Disabled);
    assert_eq!(default.radix, Radix::Hexadecimal);
    assert_eq!(default.numeric_mode, NumericMode::Integer);
    assert_eq!(
        default.word_config,
        config(16, SignedMode::TwosComplement).word_config
    );
    assert_eq!(default.flags.get(5), Ok(false));
    assert_eq!(default.flags.get(6), Err(CalcError::InvalidFlag(6)));
    for count in [0, 1025] {
        let invalid = CalculatorConfig {
            register_count: count,
            ..CalculatorConfig::default()
        };
        assert_eq!(
            Calculator::with_config(invalid).err(),
            Some(CalcError::InvalidRegisterCount(count))
        );
    }
    let invalid = CalculatorConfig {
        numeric_mode: NumericMode::FloatingPoint,
        ..CalculatorConfig::default()
    };
    assert_eq!(
        Calculator::with_config(invalid).err(),
        Some(CalcError::InvalidConfiguration)
    );

    let custom = CalculatorConfig {
        register_count: 3,
        ..config(8, SignedMode::OnesComplement)
    };
    let mut calc = Calculator::with_config(custom).unwrap();
    let original = calc.snapshot();
    let mut detached = calc.snapshot();
    detached.registers[0] = int(9, 8);
    assert_eq!(calc.snapshot(), original);
    calc.execute_all([
        Command::Push(int(7, 8)),
        Command::Store(2),
        Command::SetFlag {
            index: 3,
            value: true,
        },
        Command::SetWordSize(size(2)),
        Command::SetNumericMode(NumericMode::FloatingPoint),
    ])
    .unwrap();
    calc.reset();
    assert_eq!(calc.snapshot(), original);
}

// IA-C002–005,012 / TEST-010–014,020–031.
#[test]
fn stack_moves_and_entry_lift() {
    let mut calc = Calculator::new();
    let n = |value| int(value, 16);
    calc.execute_all([9, 8, 7, 6].map(|value| Command::Push(n(value))))
        .unwrap();
    assert_eq!(calc.snapshot().stack, [6, 7, 8, 9].map(n));
    calc.update_value(ValueUpdate::Start(n(5))).unwrap();
    assert_eq!(calc.snapshot().stack, [5, 6, 7, 8].map(n));
    calc.execute(Command::Enter).unwrap();
    assert_eq!(calc.snapshot().stack, [5, 5, 6, 7].map(n));
    calc.update_value(ValueUpdate::Start(n(4))).unwrap();
    calc.update_value(ValueUpdate::ReplaceX(n(3))).unwrap();
    assert_eq!(calc.snapshot().stack, [3, 5, 6, 7].map(n));
    assert_eq!(calc.snapshot().stack_lift, StackLift::Disabled);
    calc.execute(Command::Push(n(2))).unwrap();
    assert_eq!(calc.snapshot().stack_lift, StackLift::Enabled);
    calc.execute(Command::RollUp).unwrap();
    assert_eq!(calc.snapshot().stack, [7, 2, 5, 6].map(n));
    calc.execute(Command::RollDown).unwrap();
    calc.execute(Command::SwapXY).unwrap();
    assert_eq!(calc.snapshot().stack, [5, 2, 6, 7].map(n));
    calc.execute(Command::Subtract).unwrap();
    assert_eq!(calc.snapshot().stack, [65533, 6, 7, 7].map(n));
    assert_eq!(calc.snapshot().last_x, n(5));
    calc.execute_all([Command::ClearX, Command::RecallLastX, Command::Store(15)])
        .unwrap();
    assert_eq!(calc.snapshot().stack, [5, 6, 7, 7].map(n));
    expect_error(
        &mut calc,
        Command::Recall(16),
        CalcError::InvalidRegister(16),
    );
    calc.execute_all([Command::ClearStack, Command::Recall(15)])
        .unwrap();
    assert_eq!(calc.snapshot().stack, [5, 0, 0, 0].map(n));
    assert_eq!(calc.snapshot().last_x, n(0));
    calc.execute(Command::ClearRegisters).unwrap();
    assert_eq!(calc.snapshot().registers, vec![n(0); 16]);
}

#[test]
fn errors_have_priority_and_single_commands_are_atomic() {
    let mut calc = Calculator::with_config(config(8, SignedMode::Unsigned)).unwrap();
    expect_error(
        &mut calc,
        Command::Push(Number::Float(f64::NAN)),
        CalcError::FloatingPointOutOfRange,
    );
    expect_error(
        &mut calc,
        Command::Push(Number::Float(1.0)),
        CalcError::NumericModeMismatch,
    );
    expect_error(
        &mut calc,
        Command::Push(int(256, 16)),
        CalcError::IntegerOutOfRange,
    );
    let before = calc.snapshot();
    assert_eq!(
        calc.update_value(ValueUpdate::ReplaceX(int(256, 16))),
        Err(CalcError::IntegerOutOfRange)
    );
    assert_eq!(calc.snapshot(), before);
    assert_eq!(
        calc.execute_all([
            Command::Push(int(5, 8)),
            Command::Push(int(0, 8)),
            Command::Divide,
            Command::Push(int(7, 8))
        ]),
        Err(CalcError::DivisionByZero)
    );
    assert_eq!(calc.snapshot().stack, [0, 5, 0, 0].map(|v| int(v, 8)));
    calc.execute(Command::SetNumericMode(NumericMode::FloatingPoint))
        .unwrap();
    expect_error(
        &mut calc,
        Command::EditBit {
            index: 255,
            edit: BitEdit::Set,
        },
        CalcError::InvalidBitIndex(255),
    );
    expect_error(&mut calc, Command::MaskLeft(9), CalcError::InvalidCount);
    expect_error(&mut calc, Command::And, CalcError::NumericModeMismatch);
    expect_error(
        &mut calc,
        Command::SetRadix(Radix::Hexadecimal),
        CalcError::NumericModeMismatch,
    );
    expect_error(
        &mut calc,
        Command::SetFlag {
            index: 6,
            value: true,
        },
        CalcError::InvalidFlag(6),
    );
    assert_eq!(calc.test_bit(8), Err(CalcError::InvalidBitIndex(8)));
}

#[test]
fn integer_edge_arithmetic_and_flags() {
    let mut calc = Calculator::with_config(config(64, SignedMode::Unsigned)).unwrap();
    let n = |v| int(v, 64);
    calc.execute_all([
        Command::SetFlag {
            index: 1,
            value: true,
        },
        Command::SetFlag {
            index: 4,
            value: true,
        },
        Command::Push(n(u64::MAX)),
        Command::Push(n(u64::MAX)),
        Command::Multiply,
    ])
    .unwrap();
    assert_eq!(raw(&calc), 1);
    assert!(calc.snapshot().flags.carry());
    assert!(calc.snapshot().flags.out_of_range());
    assert_eq!(calc.snapshot().flags.get(1), Ok(true));
    calc.update_value(ValueUpdate::ReplaceX(n(u64::MAX)))
        .unwrap();
    calc.execute(Command::Sqrt).unwrap();
    assert_eq!(raw(&calc), u32::MAX.into());
    assert!(calc.snapshot().flags.carry());
    assert!(calc.snapshot().flags.out_of_range());
    calc.execute_all([
        Command::SetSignedMode(SignedMode::TwosComplement),
        Command::Push(n(1 << 63)),
        Command::Push(n(u64::MAX)),
        Command::Divide,
    ])
    .unwrap();
    assert_eq!(raw(&calc), 1 << 63);
    assert!(!calc.snapshot().flags.carry());
    assert!(calc.snapshot().flags.out_of_range());
    expect_error(&mut calc, Command::Sqrt, CalcError::DomainError);
    calc.execute(Command::Absolute).unwrap();
    assert_eq!(raw(&calc), 1 << 63);
    assert!(calc.snapshot().flags.out_of_range());
}

// Independent small-word model using signed magnitudes and modular residues.
#[test]
fn exhaustive_small_integer_binary_operations() {
    for width in 1..=4 {
        let modulus = 1_i64 << width;
        let max = modulus - 1;
        let half = modulus / 2;
        for (mode, zero_mode) in [
            (SignedMode::Unsigned, OnesZeroMode::Canonical),
            (SignedMode::TwosComplement, OnesZeroMode::Canonical),
            (SignedMode::OnesComplement, OnesZeroMode::Raw),
            (SignedMode::OnesComplement, OnesZeroMode::Canonical),
        ] {
            let ones = mode == SignedMode::OnesComplement;
            let signed = mode != SignedMode::Unsigned;
            let (min, limit) = if signed {
                (-half + i64::from(ones), half - 1)
            } else {
                (0, max)
            };
            let negative = |raw| signed && raw >= half;
            let value = |raw| {
                if negative(raw) {
                    raw - if ones { max } else { modulus }
                } else {
                    raw
                }
            };
            let encode = |value: i64, neg_zero| {
                if ones && value == 0 && neg_zero {
                    max
                } else if ones && value < 0 {
                    max + value
                } else {
                    value.rem_euclid(modulus)
                }
            };
            for y in 0..=max {
                for x in 0..=max {
                    let (a, b) = (value(y), value(x));
                    let mut calc = Calculator::with_config(config(width, mode)).unwrap();
                    calc.execute(Command::SetOnesZeroMode(zero_mode)).unwrap();
                    calc.execute_all([
                        Command::Push(int(y as u64, width)),
                        Command::Push(int(x as u64, width)),
                    ])
                    .unwrap();
                    for (command, math, result, carry, updates_range) in [
                        (
                            Command::Add,
                            a + b,
                            if ones && x + y > 0 {
                                1 + (x + y - 1) % max
                            } else {
                                (x + y) % modulus
                            },
                            Some(x + y > max),
                            true,
                        ),
                        (
                            Command::Subtract,
                            a - b,
                            if ones && y < x {
                                max - (x - y)
                            } else {
                                (y - x).rem_euclid(modulus)
                            },
                            Some(y < x),
                            true,
                        ),
                        (
                            Command::Multiply,
                            a * b,
                            if ones {
                                let magnitude = (a * b).abs() % max;
                                if negative(y) ^ negative(x) {
                                    max - magnitude
                                } else {
                                    magnitude
                                }
                            } else {
                                (a * b).rem_euclid(modulus)
                            },
                            None,
                            true,
                        ),
                        (Command::Divide, 0, 0, None, true),
                        (Command::Remainder, 0, 0, None, false),
                    ] {
                        for previous in [false, true] {
                            let mut candidate = calc.clone();
                            candidate
                                .execute_all([
                                    Command::SetFlag {
                                        index: 3,
                                        value: true,
                                    },
                                    Command::SetFlag {
                                        index: 4,
                                        value: previous,
                                    },
                                    Command::SetFlag {
                                        index: 5,
                                        value: !previous,
                                    },
                                ])
                                .unwrap();
                            let (math, result, carry) =
                                if matches!(command, Command::Divide | Command::Remainder) {
                                    if b == 0 {
                                        expect_error(
                                            &mut candidate,
                                            command,
                                            CalcError::DivisionByZero,
                                        );
                                        continue;
                                    }
                                    if command == Command::Divide {
                                        (
                                            a / b,
                                            encode(a / b, negative(y) ^ negative(x)),
                                            Some(a % b != 0),
                                        )
                                    } else {
                                        (a % b, encode(a % b, negative(y)), None)
                                    }
                                } else {
                                    (math, result, carry)
                                };
                            candidate.execute(command).unwrap();
                            let context = (width, mode, zero_mode, y, x, command);
                            let result =
                                if ones && zero_mode == OnesZeroMode::Canonical && result == max {
                                    0
                                } else {
                                    result
                                };
                            assert_eq!(raw(&candidate), result as u64, "{context:?}");
                            let snapshot = candidate.snapshot();
                            assert_eq!(snapshot.last_x, int(x as u64, width));
                            assert_eq!(snapshot.stack[1..], [int(0, width); 3]);
                            assert_eq!(
                                snapshot.flags.carry(),
                                carry.unwrap_or(previous),
                                "{context:?}"
                            );
                            assert_eq!(
                                snapshot.flags.out_of_range(),
                                if updates_range {
                                    !(min..=limit).contains(&math)
                                } else {
                                    !previous
                                },
                                "{context:?}"
                            );
                            assert_eq!(snapshot.flags.get(3), Ok(true));
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn bitwise_masks_rotation_and_width_boundaries() {
    let mut calc = Calculator::with_config(config(64, SignedMode::TwosComplement)).unwrap();
    let n = |v| int(v, 64);
    calc.execute_all([Command::MaskLeft(64), Command::CountBits])
        .unwrap();
    assert_eq!(raw(&calc), 64);
    assert_eq!(calc.snapshot().last_x, n(u64::MAX));
    calc.execute_all([
        Command::MaskRight(0),
        Command::EditBit {
            index: 63,
            edit: BitEdit::Set,
        },
    ])
    .unwrap();
    assert_eq!(calc.test_bit(63), Ok(true));
    assert_eq!(calc.test_bit(64), Err(CalcError::InvalidBitIndex(64)));
    calc.execute(Command::ShiftRightArithmetic).unwrap();
    assert_eq!(raw(&calc), 3 << 62);
    calc.execute_all([
        Command::SetSignedMode(SignedMode::Unsigned),
        Command::SetFlag {
            index: 4,
            value: true,
        },
        Command::Push(n(65)),
        Command::RotateN(RotateKind::LeftCarry),
    ])
    .unwrap();
    assert_eq!(raw(&calc), 3 << 62);
    assert!(calc.snapshot().flags.carry());
    calc.execute(Command::Push(n(66))).unwrap();
    expect_error(
        &mut calc,
        Command::RotateN(RotateKind::LeftCarry),
        CalcError::InvalidCount,
    );
    calc.update_value(ValueUpdate::ReplaceX(n(u64::MAX)))
        .unwrap();
    expect_error(
        &mut calc,
        Command::RotateN(RotateKind::Left),
        CalcError::InvalidCount,
    );

    let mut calc = Calculator::with_config(config(4, SignedMode::TwosComplement)).unwrap();
    calc.execute_all([
        Command::Push(int(9, 4)),
        Command::Push(int(14, 4)),
        Command::RotateN(RotateKind::Left),
    ])
    .unwrap();
    assert_eq!(raw(&calc), 6);
    assert_eq!(calc.snapshot().last_x, int(14, 4));
    assert!(!calc.snapshot().flags.carry());
    calc.execute_all([Command::Push(int(4, 4)), Command::RotateN(RotateKind::Left)])
        .unwrap();
    assert_eq!(raw(&calc), 6);
    assert!(!calc.snapshot().flags.carry());
    calc.execute_all([
        Command::SetFlag {
            index: 4,
            value: true,
        },
        Command::Push(int(0, 4)),
        Command::RotateN(RotateKind::RightCarry),
    ])
    .unwrap();
    assert_eq!(raw(&calc), 6);
    assert!(calc.snapshot().flags.carry());
}

#[test]
fn floating_arithmetic_preserves_flags_and_chs_history() {
    let mut calc = Calculator::new();
    calc.execute_all([
        Command::SetNumericMode(NumericMode::FloatingPoint),
        Command::SetFlag {
            index: 4,
            value: true,
        },
        Command::SetFlag {
            index: 5,
            value: true,
        },
        Command::Push(Number::Float(5.5)),
        Command::Push(Number::Float(2.0)),
        Command::Remainder,
        Command::ChangeSign,
    ])
    .unwrap();
    assert_eq!(calc.snapshot().stack[0], Number::Float(-1.5));
    assert_eq!(calc.snapshot().last_x, Number::Float(2.0));
    assert!(calc.snapshot().flags.carry());
    assert!(calc.snapshot().flags.out_of_range());
    expect_error(&mut calc, Command::Sqrt, CalcError::DomainError);
    calc.execute(Command::Push(Number::Float(-0.0))).unwrap();
    expect_error(&mut calc, Command::Divide, CalcError::DivisionByZero);
    calc.execute(Command::Absolute).unwrap();
    assert_eq!(calc.snapshot().stack[0].as_float().unwrap().to_bits(), 0);
    calc.execute_all([
        Command::Push(Number::Float(f64::MAX)),
        Command::Push(Number::Float(2.0)),
    ])
    .unwrap();
    expect_error(
        &mut calc,
        Command::Multiply,
        CalcError::FloatingPointOutOfRange,
    );
}

#[test]
fn transitions_are_exact_and_include_stack_history_and_registers() {
    let mut calc = Calculator::with_config(config(8, SignedMode::OnesComplement)).unwrap();
    calc.execute_all([
        Command::Push(int(255, 8)),
        Command::Store(2),
        Command::ChangeSign,
        Command::SetFlag {
            index: 5,
            value: true,
        },
        Command::SetNumericMode(NumericMode::FloatingPoint),
    ])
    .unwrap();
    let snapshot = calc.snapshot();
    assert_eq!(snapshot.radix, Radix::Decimal);
    assert_eq!(snapshot.stack[0].as_float().unwrap().to_bits(), 0);
    assert_eq!(
        snapshot.last_x.as_float().unwrap().to_bits(),
        (-0.0_f64).to_bits()
    );
    assert_eq!(
        snapshot.registers[2].as_float().unwrap().to_bits(),
        (-0.0_f64).to_bits()
    );
    assert!(snapshot.flags.out_of_range());
    assert_eq!(snapshot.stack_lift, StackLift::Enabled);
    calc.execute_all([
        Command::SetNumericMode(NumericMode::Integer),
        Command::SetWordSize(size(16)),
        Command::SetSignedMode(SignedMode::TwosComplement),
    ])
    .unwrap();
    assert_eq!(calc.snapshot().last_x, int(255, 16));
    assert_eq!(calc.snapshot().registers[2], int(255, 16));
    calc.execute_all([
        Command::SetWordSize(size(4)),
        Command::SetWordSize(size(16)),
    ])
    .unwrap();
    assert_eq!(calc.snapshot().last_x, int(15, 16));

    let mut calc = Calculator::with_config(config(64, SignedMode::Unsigned)).unwrap();
    for value in [u64::MAX, (1 << 53) + 1] {
        calc.execute_all([
            Command::Push(int(value, 64)),
            Command::Store(0),
            Command::ClearStack,
        ])
        .unwrap();
        expect_error(
            &mut calc,
            Command::SetNumericMode(NumericMode::FloatingPoint),
            CalcError::IntegerOutOfRange,
        );
    }
    calc.execute_all([
        Command::Push(int(1 << 63, 64)),
        Command::Store(0),
        Command::SetNumericMode(NumericMode::FloatingPoint),
    ])
    .unwrap();
    for value in [2.0_f64.powi(64), 1.5, -1.0] {
        calc.update_value(ValueUpdate::ReplaceX(Number::Float(value)))
            .unwrap();
        expect_error(
            &mut calc,
            Command::SetNumericMode(NumericMode::Integer),
            CalcError::IntegerOutOfRange,
        );
    }
}

#[test]
fn formatting_uses_raw_nondecimal_and_keeps_negative_zero() {
    let mut calc = Calculator::with_config(config(8, SignedMode::OnesComplement)).unwrap();
    calc.execute(Command::SetFlag {
        index: 3,
        value: true,
    })
    .unwrap();
    let before = calc.snapshot();
    for (radix, expected) in [
        (Radix::Binary, "00000001"),
        (Radix::Octal, "001"),
        (Radix::Decimal, "1"),
        (Radix::Hexadecimal, "01"),
    ] {
        assert_eq!(
            format_number(
                int(1, 8),
                radix,
                NumericMode::Integer,
                &before.word_config,
                &before.flags
            )
            .unwrap(),
            expected
        );
    }
    assert_eq!(
        format_number(
            int(255, 8),
            Radix::Decimal,
            NumericMode::Integer,
            &before.word_config,
            &before.flags
        )
        .unwrap(),
        "-0"
    );
    assert_eq!(
        format_number(
            int(255, 8),
            Radix::Hexadecimal,
            NumericMode::Integer,
            &before.word_config,
            &before.flags
        )
        .unwrap(),
        "FF"
    );
    for value in [-0.0_f64, 0.0, 0.1, f64::MAX, f64::from_bits(1)] {
        let formatted = format_number(
            Number::Float(value),
            Radix::Decimal,
            NumericMode::FloatingPoint,
            &before.word_config,
            &before.flags,
        )
        .unwrap();
        assert_eq!(formatted.parse::<f64>().unwrap().to_bits(), value.to_bits());
    }
    assert_eq!(
        format_number(
            Number::Float(f64::INFINITY),
            Radix::Binary,
            NumericMode::FloatingPoint,
            &before.word_config,
            &before.flags
        ),
        Err(CalcError::FloatingPointOutOfRange)
    );
    assert_eq!(calc.snapshot(), before);
}

// TEST-049/050/052/063: each command uses the original X and independent flags.
#[test]
fn logic_shift_rotate_and_edit_command_effects() {
    let n = |value| int(value, 8);
    for (command, expected, binary) in [
        (Command::And, 0x05, true),
        (Command::Or, 0xaf, true),
        (Command::Xor, 0xaa, true),
        (Command::Not, 0xf0, false),
    ] {
        for carry in [false, true] {
            for range in [false, true] {
                let mut calc =
                    Calculator::with_config(config(8, SignedMode::TwosComplement)).unwrap();
                calc.execute_all([
                    Command::Push(n(0xa5)),
                    Command::Push(n(0x0f)),
                    Command::SetFlag {
                        index: 4,
                        value: carry,
                    },
                    Command::SetFlag {
                        index: 5,
                        value: range,
                    },
                    command,
                ])
                .unwrap();
                let snapshot = calc.snapshot();
                assert_eq!(
                    snapshot.stack,
                    [expected, if binary { 0 } else { 0xa5 }, 0, 0].map(n)
                );
                assert_eq!(snapshot.last_x, n(0x0f));
                assert_eq!(snapshot.flags.carry(), carry);
                assert_eq!(snapshot.flags.out_of_range(), range);
            }
        }
    }
    for mode in [SignedMode::Unsigned, SignedMode::TwosComplement] {
        let asr = if mode == SignedMode::Unsigned {
            0x40
        } else {
            0xc0
        };
        for (command, expected) in [
            (Command::ShiftLeft, 0x02),
            (Command::ShiftRight, 0x40),
            (Command::ShiftRightArithmetic, asr),
            (Command::RotateLeft, 0x03),
            (Command::RotateRight, 0xc0),
            (Command::RotateLeftCarry, 0x02),
            (Command::RotateRightCarry, 0x40),
        ] {
            for overflow in [false, true] {
                let mut calc = Calculator::with_config(config(8, mode)).unwrap();
                calc.execute_all([
                    Command::Push(n(0x81)),
                    Command::SetFlag {
                        index: 5,
                        value: overflow,
                    },
                    command,
                ])
                .unwrap();
                let after = calc.snapshot();
                assert_eq!(raw(&calc), expected, "{mode:?} {command:?}");
                assert_eq!(after.last_x, n(0x81));
                assert!(after.flags.carry());
                assert_eq!(after.flags.out_of_range(), overflow);
            }
        }
    }
    let mut calc = Calculator::with_config(config(8, SignedMode::Unsigned)).unwrap();
    calc.execute(Command::MaskLeft(3)).unwrap();
    assert_eq!(raw(&calc), 0xe0);
    calc.execute(Command::MaskRight(3)).unwrap();
    assert_eq!(raw(&calc), 0x07);
    calc.execute(Command::EditBit {
        index: 0,
        edit: BitEdit::Clear,
    })
    .unwrap();
    let before = calc.snapshot();
    assert_eq!(raw(&calc), 0x06);
    assert_eq!(calc.test_bit(0), Ok(false));
    assert_eq!(calc.test_bit(1), Ok(true));
    assert_eq!(calc.snapshot(), before);
    calc.execute(Command::EditBit {
        index: 0,
        edit: BitEdit::Set,
    })
    .unwrap();
    assert_eq!(raw(&calc), 0x07);

    for (command, expected) in [(Command::ShiftLeft, 0), (Command::RotateLeft, 1)] {
        let mut calc = Calculator::with_config(config(1, SignedMode::Unsigned)).unwrap();
        calc.execute_all([Command::Push(int(1, 1)), command])
            .unwrap();
        assert_eq!(raw(&calc), expected);
        assert!(calc.snapshot().flags.carry());
    }
}

// IA-C011,013. Construct the documented initial stack and LAST X through public commands.
#[test]
fn batch_failure_preserves_exact_specification_fixture() {
    let mut calc = Calculator::with_config(CalculatorConfig {
        radix: Radix::Decimal,
        ..config(8, SignedMode::TwosComplement)
    })
    .unwrap();
    let n = |value| int(value, 8);
    calc.execute_all([
        Command::Push(n(9)),
        Command::ChangeSign,
        Command::ClearX,
        Command::Push(n(4)),
        Command::Push(n(3)),
        Command::Push(n(2)),
        Command::Push(n(5)),
    ])
    .unwrap();
    assert_eq!(calc.snapshot().stack, [5, 2, 3, 4].map(n));
    assert_eq!(calc.snapshot().last_x, n(9));

    assert_eq!(
        calc.execute_all([
            Command::Push(n(5)),
            Command::Push(n(0)),
            Command::Divide,
            Command::Push(n(7)),
        ]),
        Err(CalcError::DivisionByZero)
    );
    assert_eq!(calc.snapshot().stack, [0, 5, 5, 2].map(n));
    assert_eq!(calc.snapshot().last_x, n(9));
    assert_eq!(calc.snapshot().stack_lift, StackLift::Enabled);
    expect_error(&mut calc, Command::Remainder, CalcError::DivisionByZero);
}

#[test]
fn unary_arithmetic_and_float_subnormals() {
    for (mode, input, command, expected, out_of_range) in [
        (
            SignedMode::TwosComplement,
            0xfb,
            Command::Absolute,
            5,
            false,
        ),
        (
            SignedMode::TwosComplement,
            0x80,
            Command::ChangeSign,
            0x80,
            true,
        ),
        (SignedMode::Unsigned, 0xfb, Command::Absolute, 0xfb, false),
        (SignedMode::Unsigned, 0, Command::ChangeSign, 0, false),
        (SignedMode::Unsigned, 1, Command::ChangeSign, 0xff, true),
        (
            SignedMode::OnesComplement,
            0xff,
            Command::Absolute,
            0,
            false,
        ),
        (SignedMode::OnesComplement, 0xff, Command::Sqrt, 0, false),
    ] {
        let mut calc = Calculator::with_config(config(8, mode)).unwrap();
        calc.execute_all([Command::Push(int(input, 8)), command])
            .unwrap();
        assert_eq!(raw(&calc), expected);
        assert_eq!(calc.snapshot().flags.out_of_range(), out_of_range);
        assert_eq!(calc.snapshot().last_x, int(input, 8));
    }
    let mut calc = Calculator::with_config(config(8, SignedMode::Unsigned)).unwrap();
    calc.execute_all([
        Command::SetFlag {
            index: 4,
            value: true,
        },
        Command::Push(int(16, 8)),
        Command::Sqrt,
    ])
    .unwrap();
    assert_eq!(raw(&calc), 4);
    assert!(!calc.snapshot().flags.carry());

    calc.execute_all([
        Command::ClearStack,
        Command::SetNumericMode(NumericMode::FloatingPoint),
        Command::Push(Number::Float(0.5)),
        Command::Push(Number::Float(0.25)),
        Command::Add,
    ])
    .unwrap();
    assert_eq!(calc.snapshot().stack[0], Number::Float(0.75));
    let tiny = f64::from_bits(1);
    calc.execute_all([
        Command::Push(Number::Float(tiny)),
        Command::Push(Number::Float(2.0)),
        Command::Divide,
    ])
    .unwrap();
    assert_eq!(calc.snapshot().stack[0].as_float().unwrap().to_bits(), 0);
    assert_eq!(calc.snapshot().last_x, Number::Float(2.0));
}

#[test]
fn oneszero_switch_preserves_state_and_reset_restores_creation_mode() {
    use Command::*;
    let mut initial = config(64, SignedMode::OnesComplement);
    initial.word_config.ones_zero_mode = OnesZeroMode::Raw;
    let mut calc = Calculator::with_config(initial).unwrap();
    calc.execute_all([
        Push(int(1, 64)),
        Push(int(u64::MAX - 1, 64)),
        Add,
        Store(0),
        Enter,
    ])
    .unwrap();
    assert_eq!(raw(&calc), u64::MAX);
    let mut expected = calc.snapshot();
    expected.word_config.ones_zero_mode = OnesZeroMode::Canonical;
    calc.execute(SetOnesZeroMode(OnesZeroMode::Canonical))
        .unwrap();
    assert_eq!(calc.snapshot(), expected);
    calc.execute_all([Push(int(1, 64)), Push(int(u64::MAX - 1, 64)), Add])
        .unwrap();
    assert_eq!(raw(&calc), 0);
    calc.reset();
    assert_eq!(
        calc.snapshot(),
        Calculator::with_config(initial).unwrap().snapshot()
    );
}

#[test]
fn oneszero_can_be_selected_in_float_and_unsigned_modes() {
    use Command::*;
    let mut calc = Calculator::new();
    assert_eq!(
        calc.snapshot().word_config.ones_zero_mode,
        OnesZeroMode::Canonical
    );
    calc.execute_all([
        SetNumericMode(NumericMode::FloatingPoint),
        SetOnesZeroMode(OnesZeroMode::Raw),
        Push(Number::Float(-0.0)),
        Push(Number::Float(2.0)),
        Multiply,
    ])
    .unwrap();
    assert_eq!(
        calc.snapshot().stack[0].as_float().unwrap().to_bits(),
        (-0.0_f64).to_bits()
    );
    calc.execute_all([
        ClearStack,
        SetNumericMode(NumericMode::Integer),
        SetSignedMode(SignedMode::Unsigned),
        SetOnesZeroMode(OnesZeroMode::Canonical),
        Push(int(0xffff, 16)),
        Push(int(0, 16)),
        Add,
    ])
    .unwrap();
    assert_eq!(raw(&calc), 0xffff);
    calc.execute_all([
        SetOnesZeroMode(OnesZeroMode::Raw),
        SetSignedMode(SignedMode::OnesComplement),
        Push(int(1, 16)),
        Push(int(0xfffe, 16)),
        Add,
    ])
    .unwrap();
    assert_eq!(raw(&calc), 0xffff);
}

#[test]
fn documented_word_and_stack_boundaries() {
    use Command::*;
    use SignedMode::*;
    assert_eq!(Word::from_bits(0x12a5, size(8)).bits(), 0xa5);
    for (bits, mode, cases) in [
        (
            8,
            TwosComplement,
            &[(0x80, -128), (0xff, -1), (0x7f, 127)][..],
        ),
        (1, Unsigned, &[(0, 0), (1, 1)]),
        (1, TwosComplement, &[(0, 0), (1, -1)]),
        (1, OnesComplement, &[(0, 0), (1, 0)]),
    ] {
        for &(raw, expected) in cases {
            let word = Word::from_bits(raw, size(bits));
            let cfg = config(bits, mode).word_config;
            assert_eq!(word.to_signed(&cfg), expected);
            assert_eq!(
                word.is_negative_zero(&cfg),
                mode == OnesComplement && raw == 1
            );
        }
    }
    let mut c = Calculator::new();
    c.execute(Add).unwrap();
    assert_eq!(c.snapshot().stack, [int(0, 16); 4]);
    expect_error(&mut c, Store(16), CalcError::InvalidRegister(16));
    let mut c = Calculator::with_config(config(64, Unsigned)).unwrap();
    c.execute(Push(int(u64::MAX, 64))).unwrap();
    expect_error(
        &mut c,
        SetNumericMode(NumericMode::FloatingPoint),
        CalcError::IntegerOutOfRange,
    );
    c.execute(SetWordSize(size(8))).unwrap();
    c.execute(CountBits).unwrap();
    assert_eq!(raw(&c), 8);
}

#[test]
fn eight_bit_arithmetic_result_flags_and_history() {
    use Command::*;
    use SignedMode::*;
    for (mode, y, x, op, result, new_c, new_g) in [
        (Unsigned, 255, 1, Add, 0, Some(true), Some(true)),
        (TwosComplement, 127, 1, Add, 128, Some(false), Some(true)),
        (TwosComplement, 255, 255, Add, 254, Some(true), Some(false)),
        (Unsigned, 2, 3, Multiply, 6, None, Some(false)),
        (TwosComplement, 249, 3, Divide, 254, Some(true), Some(false)),
        (TwosComplement, 249, 3, Remainder, 255, None, None),
        (
            TwosComplement,
            128,
            255,
            Divide,
            128,
            Some(false),
            Some(true),
        ),
        (TwosComplement, 7, 4, Remainder, 3, None, None),
    ] {
        for carry in [false, true] {
            for overflow in [false, true] {
                let mut calc = Calculator::with_config(config(8, mode)).unwrap();
                calc.execute_all([
                    Push(int(42, 8)),
                    Push(int(y, 8)),
                    Push(int(x, 8)),
                    SetFlag {
                        index: 4,
                        value: carry,
                    },
                    SetFlag {
                        index: 5,
                        value: overflow,
                    },
                    op,
                ])
                .unwrap();
                let s = calc.snapshot();
                assert_eq!(s.stack, [result, 42, 0, 0].map(|v| int(v, 8)), "{op:?}");
                assert_eq!(s.last_x, int(x, 8));
                assert_eq!(s.flags.carry(), new_c.unwrap_or(carry));
                assert_eq!(s.flags.out_of_range(), new_g.unwrap_or(overflow));
            }
        }
    }
}

#[test]
fn absolute_and_rotate_count_boundaries() {
    use Command::*;
    for (mode, input, result, overflow) in [
        (SignedMode::TwosComplement, 251, 5, false),
        (SignedMode::TwosComplement, 128, 128, true),
        (SignedMode::Unsigned, 251, 251, false),
    ] {
        for carry in [false, true] {
            let mut c = Calculator::with_config(config(8, mode)).unwrap();
            c.execute_all([
                Push(int(input, 8)),
                SetFlag {
                    index: 4,
                    value: carry,
                },
                Absolute,
            ])
            .unwrap();
            assert_eq!(raw(&c), result);
            assert_eq!(c.snapshot().last_x, int(input, 8));
            assert_eq!(c.snapshot().flags.carry(), carry);
            assert_eq!(c.snapshot().flags.out_of_range(), overflow);
        }
    }
    for (kind, count, expected) in [
        (RotateKind::Left, 0, Some(0x81)),
        (RotateKind::Left, 255, Some(3)),
        (RotateKind::Left, 8, Some(0x81)),
        (RotateKind::Left, 9, None),
        (RotateKind::LeftCarry, 9, Some(0x81)),
        (RotateKind::LeftCarry, 10, None),
    ] {
        let mut c = Calculator::with_config(config(8, SignedMode::TwosComplement)).unwrap();
        c.execute_all([
            Push(int(42, 8)),
            Push(int(0x81, 8)),
            Push(int(count, 8)),
            SetFlag {
                index: 4,
                value: true,
            },
        ])
        .unwrap();
        if let Some(value) = expected {
            c.execute(RotateN(kind)).unwrap();
            assert_eq!(c.snapshot().stack, [value, 42, 0, 0].map(|v| int(v, 8)));
            assert!(c.snapshot().flags.carry());
        } else {
            expect_error(&mut c, RotateN(kind), CalcError::InvalidCount);
        }
    }
}

#[test]
fn core_entry_api_uses_the_common_acceptance_fixture() {
    use Command::*;
    let n = |v| int(v, 8);
    let mut base = Calculator::with_config(CalculatorConfig {
        radix: Radix::Decimal,
        ..config(8, SignedMode::TwosComplement)
    })
    .unwrap();
    base.execute_all([
        Push(n(9)),
        Absolute,
        Push(n(7)),
        Store(0),
        Push(n(4)),
        Push(n(3)),
        Push(n(2)),
        Push(n(5)),
    ])
    .unwrap();
    for index in [0, 2, 4, 5] {
        base.execute(SetFlag { index, value: true }).unwrap();
    }
    let initial = base.snapshot();
    assert_eq!(initial.stack, [5, 2, 3, 4].map(n));
    assert_eq!(initial.last_x, n(9));
    for enter in [false, true] {
        for edit in [false, true] {
            let mut c = base.clone();
            if enter {
                c.execute(Enter).unwrap();
            }
            if edit {
                c.update_value(ValueUpdate::Start(n(1))).unwrap();
                c.update_value(ValueUpdate::ReplaceX(n(12))).unwrap();
            } else {
                c.execute(Push(n(3))).unwrap();
            }
            let mut expected = initial.clone();
            expected.stack = [if edit { 12 } else { 3 }, 5, 2, 3].map(n);
            if edit && enter {
                expected.stack_lift = StackLift::Disabled;
            }
            assert_eq!(c.snapshot(), expected);
        }
    }
    expect_error(
        &mut base,
        Push(int(0x123, 16)),
        CalcError::IntegerOutOfRange,
    );
    expect_error(
        &mut base,
        Push(Number::Float(f64::NAN)),
        CalcError::FloatingPointOutOfRange,
    );
    expect_error(
        &mut base,
        Push(Number::Float(1.0)),
        CalcError::NumericModeMismatch,
    );
    // TEST-048 skips Push(9), IA-C013 skips Push(7).
    for skipped in [9, 7] {
        let mut batch = base.clone();
        assert_eq!(
            batch.execute_all([Push(n(5)), Push(n(0)), Divide, Push(n(skipped))]),
            Err(CalcError::DivisionByZero)
        );
        let mut committed = initial.clone();
        committed.stack = [0, 5, 5, 2].map(n);
        assert_eq!(batch.snapshot(), committed);
    }
    let mut detached = base.snapshot();
    detached.stack[0] = n(0);
    detached.registers[0] = n(0);
    detached.flags = Flags::default();
    assert_ne!(detached, initial);
    assert_eq!(base.snapshot(), initial);
    base.execute(Push(n(0))).unwrap();
    for command in [Divide, Remainder] {
        expect_error(&mut base, command, CalcError::DivisionByZero);
    }
}

#[test]
fn widen_twos_words_without_sign_extension() {
    use Command::*;
    let mut calc = Calculator::with_config(config(8, SignedMode::TwosComplement)).unwrap();
    let n = int(255, 8);
    calc.execute_all([
        Push(n),
        Absolute,
        Push(n),
        Store(0),
        Push(n),
        Push(n),
        Push(n),
    ])
    .unwrap();
    calc.execute(SetWordSize(size(16))).unwrap();
    let snap = calc.snapshot();
    assert_eq!(snap.stack, [int(255, 16); 4]);
    assert_eq!(snap.last_x, int(255, 16));
    assert_eq!(snap.registers[0], int(255, 16));
}

#[test]
fn nonfinite_float_push_preserves_state() {
    let mut c = Calculator::new();
    c.execute(Command::SetNumericMode(NumericMode::FloatingPoint))
        .unwrap();
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        expect_error(
            &mut c,
            Command::Push(Number::Float(value)),
            CalcError::FloatingPointOutOfRange,
        );
    }
}

#[test]
fn integer_format_width_radix_and_padding_matrix() {
    for width in [1, 3, 4, 8, 13, 63, 64] {
        let mask = u64::MAX >> (64 - width);
        for mode in [
            SignedMode::Unsigned,
            SignedMode::TwosComplement,
            SignedMode::OnesComplement,
        ] {
            let mut calc = Calculator::with_config(config(width, mode)).unwrap();
            for value in [0, 1, mask] {
                calc.execute(Command::Push(int(value, width))).unwrap();
                for pad in [false, true] {
                    calc.execute(Command::SetFlag {
                        index: 3,
                        value: pad,
                    })
                    .unwrap();
                    for radix in [
                        Radix::Binary,
                        Radix::Octal,
                        Radix::Hexadecimal,
                        Radix::Decimal,
                    ] {
                        calc.execute(Command::SetRadix(radix)).unwrap();
                        let before = calc.snapshot();
                        let expected = match radix {
                            Radix::Decimal
                                if value == mask && mode == SignedMode::OnesComplement =>
                            {
                                "-0".into()
                            }
                            Radix::Decimal
                                if value == mask && mode == SignedMode::TwosComplement =>
                            {
                                "-1".into()
                            }
                            Radix::Decimal => value.to_string(),
                            _ => {
                                let repr = |v: u64| match radix {
                                    Radix::Binary => format!("{v:b}"),
                                    Radix::Octal => format!("{v:o}"),
                                    Radix::Hexadecimal => format!("{v:X}"),
                                    Radix::Decimal => unreachable!(),
                                };
                                let digits = repr(value);
                                let len = if pad { repr(mask).len() } else { digits.len() };
                                format!("{digits:0>len$}")
                            }
                        };
                        assert_eq!(
                            format_number(
                                before.stack[0],
                                radix,
                                before.numeric_mode,
                                &before.word_config,
                                &before.flags
                            )
                            .unwrap(),
                            expected,
                            "W{width} {value} {mode:?} {radix:?} F3={pad}"
                        );
                        assert_eq!(calc.snapshot(), before);
                    }
                }
            }
        }
    }
}

#[test]
fn word_boundary_addition_matches_signed_range_and_raw_carry() {
    for width in [1, 2, 4, 8, 16, 32, 63, 64] {
        let modulus = 1_i128 << width;
        let mask = modulus - 1;
        let sign = modulus / 2;
        for signed in [
            SignedMode::Unsigned,
            SignedMode::TwosComplement,
            SignedMode::OnesComplement,
        ] {
            let ones = signed == SignedMode::OnesComplement;
            let (min, max) = match signed {
                SignedMode::Unsigned => (0, mask),
                SignedMode::TwosComplement => (-sign, sign - 1),
                SignedMode::OnesComplement => (1 - sign, sign - 1),
            };
            let decode = |v| {
                if v < sign || signed == SignedMode::Unsigned {
                    v
                } else if ones {
                    v - mask
                } else {
                    v - modulus
                }
            };
            for zero_mode in [OnesZeroMode::Canonical, OnesZeroMode::Raw] {
                let mut cfg = config(width, signed);
                cfg.word_config.ones_zero_mode = zero_mode;
                let mut calc = Calculator::with_config(cfg).unwrap();
                for y in [0, 1, sign - 1, sign, mask - 1, mask] {
                    for x in [0, 1, sign - 1, sign, mask - 1, mask] {
                        let sum = y + x;
                        let carry = sum > mask;
                        let mut expected = (sum & mask) + i128::from(ones && carry);
                        if ones && zero_mode == OnesZeroMode::Canonical && expected == mask {
                            expected = 0;
                        }
                        calc.execute_all([
                            Command::Push(int(y as u64, width)),
                            Command::Push(int(x as u64, width)),
                            Command::Add,
                        ])
                        .unwrap();
                        let s = calc.snapshot();
                        assert_eq!(
                            raw(&calc),
                            expected as u64,
                            "W{width} {signed:?} {zero_mode:?} {y}+{x}"
                        );
                        assert_eq!(s.flags.carry(), carry);
                        assert_eq!(
                            s.flags.out_of_range(),
                            !(min..=max).contains(&(decode(y) + decode(x)))
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn mask_edit_count_and_zero_rotation_keep_flags() {
    use Command::*;
    for width in [1, 8, 64] {
        let mask = u64::MAX >> (64 - width);
        let high = 1u64 << (width - 1);
        for carry in [false, true] {
            for range in [false, true] {
                let mut c = Calculator::with_config(config(width, SignedMode::Unsigned)).unwrap();
                c.execute_all([
                    SetFlag {
                        index: 4,
                        value: carry,
                    },
                    SetFlag {
                        index: 5,
                        value: range,
                    },
                ])
                .unwrap();
                let flags = c.snapshot().flags;
                for (op, expected) in [
                    (MaskLeft(0), 0),
                    (MaskLeft(width), mask),
                    (MaskRight(0), 0),
                    (MaskRight(width), mask),
                    (
                        EditBit {
                            index: width - 1,
                            edit: BitEdit::Clear,
                        },
                        mask ^ high,
                    ),
                    (
                        EditBit {
                            index: width - 1,
                            edit: BitEdit::Set,
                        },
                        mask,
                    ),
                    (CountBits, u64::from(width)),
                ] {
                    c.execute(op).unwrap();
                    assert_eq!(raw(&c), expected, "W{width} {op:?}");
                    assert_eq!(c.snapshot().flags, flags);
                }
                for kind in [
                    RotateKind::Left,
                    RotateKind::Right,
                    RotateKind::LeftCarry,
                    RotateKind::RightCarry,
                ] {
                    c.execute_all([Push(int(high, width)), Push(int(0, width)), RotateN(kind)])
                        .unwrap();
                    assert_eq!(raw(&c), high);
                    assert_eq!(c.snapshot().flags, flags);
                }
            }
        }
    }
}
