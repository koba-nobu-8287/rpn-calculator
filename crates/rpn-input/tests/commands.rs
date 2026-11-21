use rpn_core::*;
use rpn_input::*;

// Exercise the adapter against an independent, already-finalized core.
// Nonzero stack, LAST X, flags and register make accidental changes observable.
#[test]
fn finish_before_command_preserves_full_core_contract() {
    for enabled in [false, true] {
        let mut core = Calculator::new();
        let width = core.snapshot().word_config.size;
        let n = |v| Number::Integer(Word::from_bits(v, width));
        core.execute_all([
            Command::Push(n(9)),
            Command::ChangeSign,
            Command::ClearX,
            Command::Push(n(4)),
            Command::Push(n(3)),
            Command::Push(n(2)),
            Command::Store(0),
            Command::SetFlag {
                index: 3,
                value: true,
            },
        ])
        .unwrap();
        if !enabled {
            core.execute(Command::Enter).unwrap();
        }
        let mut editing = HpInputSession::new(core.clone());
        editing.apply(InputEvent::Key(Key::Digit('5'))).unwrap();
        core.update_value(ValueUpdate::Start(n(5))).unwrap();
        assert_eq!(editing.snapshot().calculator, core.snapshot());
        use Command::*;
        for cmd in [
            Enter,
            ClearX,
            ClearStack,
            ClearRegisters,
            SwapXY,
            RollUp,
            RollDown,
            RecallLastX,
            Store(0),
            Recall(0),
            Add,
            Subtract,
            Multiply,
            Divide,
            Remainder,
            ChangeSign,
            Absolute,
            Sqrt,
            And,
            Or,
            Xor,
            Not,
            ShiftLeft,
            ShiftRight,
            ShiftRightArithmetic,
            RotateLeft,
            RotateRight,
            RotateLeftCarry,
            RotateRightCarry,
            MaskLeft(3),
            MaskRight(3),
            CountBits,
            SetWordSize(WordSize::new(4).unwrap()),
            SetSignedMode(SignedMode::Unsigned),
            SetNumericMode(NumericMode::FloatingPoint),
            SetRadix(Radix::Decimal),
            SetFlag {
                index: 4,
                value: true,
            },
            Recall(16),
            Store(16),
        ] {
            let mut actual = editing.clone();
            let mut expected = core.clone();
            let result = expected.execute(cmd);
            assert_eq!(
                actual.apply(InputEvent::Command(cmd)),
                result.clone().map_err(SessionError::Calculation),
                "{cmd:?}"
            );
            assert_eq!(actual.snapshot().calculator, expected.snapshot(), "{cmd:?}");
            let buffer = if result.is_err() || cmd == ClearRegisters {
                Some("5")
            } else {
                None
            };
            assert_eq!(actual.snapshot().input_buffer.as_deref(), buffer, "{cmd:?}");
        }
    }
}
