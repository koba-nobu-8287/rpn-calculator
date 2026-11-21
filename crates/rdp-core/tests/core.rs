use rdp_core::*;

fn size(bits: u8) -> WordSize {
    WordSize::new(bits).unwrap()
}

fn int(raw: u64, bits: u8) -> Number {
    Number::Integer(Word::from_bits(raw, size(bits)))
}

fn config(bits: u8, signed_mode: SignedMode) -> CalculatorConfig {
    CalculatorConfig {
        word_config: WordConfig {
            size: size(bits),
            signed_mode,
        },
        ..CalculatorConfig::default()
    }
}

fn raw(calc: &Calculator) -> u64 {
    calc.snapshot().stack[0].as_integer().unwrap().bits()
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
        for (mode, zero_mode) in [(SignedMode::Unsigned, OnesZeroMode::Canonical), (SignedMode::TwosComplement, OnesZeroMode::Canonical), (SignedMode::OnesComplement, OnesZeroMode::Raw), (SignedMode::OnesComplement, OnesZeroMode::Canonical)] {
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
                            let result = if ones && zero_mode == OnesZeroMode::Canonical && result == max { 0 } else { result };
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
    for (mode, command, expected) in [
        (SignedMode::TwosComplement, Command::ShiftLeft, 0x02),
        (SignedMode::TwosComplement, Command::ShiftRight, 0x40),
        (
            SignedMode::TwosComplement,
            Command::ShiftRightArithmetic,
            0xc0,
        ),
        (SignedMode::Unsigned, Command::ShiftRightArithmetic, 0x40),
        (SignedMode::TwosComplement, Command::RotateLeft, 0x03),
        (SignedMode::TwosComplement, Command::RotateRight, 0xc0),
        (SignedMode::TwosComplement, Command::RotateLeftCarry, 0x02),
        (SignedMode::TwosComplement, Command::RotateRightCarry, 0x40),
    ] {
        let mut calc = Calculator::with_config(config(8, mode)).unwrap();
        calc.execute_all([
            Command::Push(n(0x81)),
            Command::SetFlag {
                index: 5,
                value: true,
            },
            command,
        ])
        .unwrap();
        assert_eq!(raw(&calc), expected, "{command:?}");
        assert_eq!(calc.snapshot().last_x, n(0x81));
        assert!(calc.snapshot().flags.carry());
        assert!(calc.snapshot().flags.out_of_range());
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
    calc.execute_all([Push(int(1, 64)), Push(int(u64::MAX - 1, 64)), Add, Store(0), Enter]).unwrap();
    assert_eq!(raw(&calc), u64::MAX);
    let mut expected = calc.snapshot();
    expected.word_config.ones_zero_mode = OnesZeroMode::Canonical;
    calc.execute(SetOnesZeroMode(OnesZeroMode::Canonical)).unwrap();
    assert_eq!(calc.snapshot(), expected);
    calc.execute_all([Push(int(1, 64)), Push(int(u64::MAX - 1, 64)), Add]).unwrap();
    assert_eq!(raw(&calc), 0);
    calc.reset();
    assert_eq!(calc.snapshot(), Calculator::with_config(initial).unwrap().snapshot());
}

#[test]
fn oneszero_can_be_selected_in_float_and_unsigned_modes() {
    use Command::*;
    let mut calc = Calculator::new();
    assert_eq!(calc.snapshot().word_config.ones_zero_mode, OnesZeroMode::Canonical);
    calc.execute_all([SetNumericMode(NumericMode::FloatingPoint), SetOnesZeroMode(OnesZeroMode::Raw),
        Push(Number::Float(-0.0)), Push(Number::Float(2.0)), Multiply]).unwrap();
    assert_eq!(calc.snapshot().stack[0].as_float().unwrap().to_bits(), (-0.0_f64).to_bits());
    calc.execute_all([ClearStack, SetNumericMode(NumericMode::Integer), SetSignedMode(SignedMode::Unsigned),
        SetOnesZeroMode(OnesZeroMode::Canonical), Push(int(0xffff, 16)), Push(int(0, 16)), Add]).unwrap();
    assert_eq!(raw(&calc), 0xffff);
    calc.execute_all([SetOnesZeroMode(OnesZeroMode::Raw), SetSignedMode(SignedMode::OnesComplement),
        Push(int(1, 16)), Push(int(0xfffe, 16)), Add]).unwrap();
    assert_eq!(raw(&calc), 0xffff);
}
