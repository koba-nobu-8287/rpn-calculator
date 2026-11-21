//! Session-level acceptance cases from rpn-calc-stack-input-test-cases.md, sections 2–3.
use rpn_core::{Command::*, NumericMode::*, StackLift::*, *};
use rpn_input::*;

#[derive(Clone, Copy)]
struct State {
    stack: [i16; 4],
    last_x: i16,
    lift: StackLift,
    buffer: Option<&'static str>,
    mode: NumericMode,
    r0: i16,
}

impl State {
    fn new(stack: [i16; 4], lift: StackLift) -> Self {
        Self {
            stack,
            last_x: 9,
            lift,
            buffer: None,
            mode: Integer,
            r0: 0,
        }
    }
    fn editing(mut self, buffer: &'static str) -> Self {
        self.buffer = Some(buffer);
        self
    }
    fn history(mut self, last_x: i16) -> Self {
        self.last_x = last_x;
        self
    }
    fn float(mut self) -> Self {
        self.mode = FloatingPoint;
        self
    }
    fn register(mut self, r0: i16) -> Self {
        self.r0 = r0;
        self
    }
    fn number(self, n: i16) -> Number {
        match self.mode {
            Integer => integer(n),
            FloatingPoint => Number::Float(f64::from(n)),
        }
    }

    fn session(self, carry: bool, overflow: bool) -> HpInputSession {
        let mut core = Calculator::with_config(CalculatorConfig {
            radix: Radix::Decimal,
            ..CalculatorConfig::default()
        })
        .unwrap();
        // ABS records the operand in LAST X. Initialize flags after arithmetic.
        core.execute_all([
            Push(integer(self.last_x)),
            Absolute,
            Push(integer(self.r0)),
            Store(0),
            SetNumericMode(self.mode),
        ])
        .unwrap();
        for (index, value) in [true, false, true, false, carry, overflow]
            .into_iter()
            .enumerate()
        {
            core.execute(SetFlag {
                index: index as u8,
                value,
            })
            .unwrap();
        }
        let [x, y, z, t] = self.stack;
        for n in [t, z, y] {
            core.execute(Push(self.number(n))).unwrap();
        }
        if self.lift == Disabled {
            core.execute(Enter).unwrap();
        }
        if self.buffer.is_none() {
            core.update_value(ValueUpdate::Start(self.number(x)))
                .unwrap();
        }
        let mut session = HpInputSession::new(core);
        if let Some(buffer) = self.buffer {
            for ch in buffer.chars() {
                let key = match ch {
                    'e' => Key::Exponent,
                    '-' => Key::ChangeSign,
                    digit => Key::Digit(digit),
                };
                session.apply(InputEvent::Key(key)).unwrap();
            }
        }
        session
    }

    #[track_caller]
    fn check(self, session: &HpInputSession, carry: bool, overflow: bool) {
        let snapshot = session.snapshot();
        let core = snapshot.calculator;
        let mut registers = vec![self.number(0); 16];
        registers[0] = self.number(self.r0);
        assert_eq!(core.stack, self.stack.map(|n| self.number(n)));
        if self.mode == FloatingPoint {
            assert_eq!(
                core.stack[0].as_float().unwrap().to_bits(),
                f64::from(self.stack[0]).to_bits()
            );
        }
        assert_eq!(core.last_x, self.number(self.last_x));
        assert_eq!(core.registers, registers);
        assert_eq!(
            core.word_config,
            WordConfig {
                ones_zero_mode: rpn_core::OnesZeroMode::Canonical,
                size: WordSize::new(16).unwrap(),
                signed_mode: SignedMode::TwosComplement
            }
        );
        assert_eq!(core.numeric_mode, self.mode);
        assert_eq!(core.radix, Radix::Decimal);
        assert_eq!(core.stack_lift, self.lift);
        assert_eq!(
            [0, 1, 2, 3, 4, 5].map(|i| core.flags.get(i).unwrap()),
            [true, false, true, false, carry, overflow]
        );
        assert_eq!(snapshot.input_buffer.as_deref(), self.buffer);
        assert_eq!(
            snapshot.entry_phase,
            if self.buffer.is_some() {
                EntryPhase::Editing
            } else {
                EntryPhase::Idle
            }
        );
        assert_eq!(
            snapshot.entry_part,
            if self.buffer.is_some_and(|s| s.contains('e')) {
                EntryPart::Exponent
            } else {
                EntryPart::Mantissa
            }
        );
    }
}

fn integer(n: i16) -> Number {
    Number::Integer(Word::from_bits(
        u64::from(n as u16),
        WordSize::new(16).unwrap(),
    ))
}

#[derive(Clone, Copy)]
enum FlagResult {
    Keep,
    Arithmetic(bool, bool),
}

#[track_caller]
fn verify(initial: State, expected: State, flags: FlagResult, apply: impl Fn(&mut HpInputSession)) {
    let combinations: &[_] = match flags {
        FlagResult::Keep => &[(false, false), (false, true), (true, false), (true, true)],
        FlagResult::Arithmetic(_, _) => &[(true, true)],
    };
    for &(carry, overflow) in combinations {
        let mut session = initial.session(carry, overflow);
        initial.check(&session, carry, overflow);
        apply(&mut session);
        let (c, g) = match flags {
            FlagResult::Keep => (carry, overflow),
            FlagResult::Arithmetic(c, g) => (c, g),
        };
        expected.check(&session, c, g);
    }
}

#[track_caller]
fn command(session: &mut HpInputSession, cmd: Command) {
    session.apply(InputEvent::Command(cmd)).unwrap();
}

#[track_caller]
fn digit(session: &mut HpInputSession, ch: char) {
    session.apply(InputEvent::Key(Key::Digit(ch))).unwrap();
}

#[test]
fn si_t001_enter() {
    verify(
        State::new([1, 2, 3, 4], Enabled),
        State::new([1, 1, 2, 3], Disabled),
        FlagResult::Keep,
        |s| {
            command(s, Enter);
        },
    );
}

#[test]
fn si_t002_repeated_enter() {
    verify(
        State::new([1, 2, 3, 4], Disabled),
        State::new([1, 1, 1, 1], Disabled),
        FlagResult::Keep,
        |s| {
            for _ in 0..3 {
                command(s, Enter);
            }
        },
    );
}

#[test]
fn si_t003_editing_enter() {
    verify(
        State::new([12, 2, 3, 4], Enabled).editing("12"),
        State::new([12, 12, 2, 3], Disabled),
        FlagResult::Keep,
        |s| {
            command(s, Enter);
        },
    );
}

#[test]
fn si_t004_digit_after_enter() {
    verify(
        State::new([5, 2, 3, 4], Disabled).editing("5"),
        State::new([3, 5, 2, 3], Disabled).editing("3"),
        FlagResult::Keep,
        |s| {
            command(s, Enter);
            digit(s, '3');
        },
    );
}

#[test]
fn si_t005_start_with_lift() {
    verify(
        State::new([8, 2, 3, 4], Enabled),
        State::new([12, 8, 2, 3], Enabled).editing("12"),
        FlagResult::Keep,
        |s| {
            digit(s, '1');
            digit(s, '2');
        },
    );
}

#[test]
fn si_t006_start_without_lift() {
    verify(
        State::new([8, 2, 3, 4], Disabled),
        State::new([12, 2, 3, 4], Disabled).editing("12"),
        FlagResult::Keep,
        |s| {
            digit(s, '1');
            digit(s, '2');
        },
    );
}

#[test]
fn si_t007_editing_add() {
    verify(
        State::new([3, 5, 2, 4], Disabled).editing("3"),
        State::new([8, 2, 4, 4], Enabled).history(3),
        FlagResult::Arithmetic(false, false),
        |s| {
            command(s, Add);
        },
    );
}

#[test]
fn si_t008_subtract() {
    verify(
        State::new([3, 5, 2, 4], Disabled),
        State::new([2, 2, 4, 4], Enabled).history(3),
        FlagResult::Arithmetic(false, false),
        |s| {
            command(s, Subtract);
        },
    );
}

#[test]
fn si_t009_repeated_drop() {
    verify(
        State::new([2, 10, 20, 30], Enabled),
        State::new([62, 30, 30, 30], Enabled).history(32),
        FlagResult::Arithmetic(false, false),
        |s| {
            for _ in 0..3 {
                command(s, Add);
            }
        },
    );
}

#[test]
fn si_t010_digit_after_add() {
    verify(
        State::new([3, 5, 2, 4], Disabled),
        State::new([7, 8, 2, 4], Enabled).editing("7").history(3),
        FlagResult::Arithmetic(false, false),
        |s| {
            command(s, Add);
            digit(s, '7');
        },
    );
}

#[test]
fn si_t011_absolute() {
    verify(
        State::new([-5, 2, 3, 4], Disabled),
        State::new([5, 2, 3, 4], Enabled).history(-5),
        FlagResult::Arithmetic(true, false),
        |s| {
            command(s, Absolute);
        },
    );
}

#[test]
fn si_t012_roll_down() {
    verify(
        State::new([1, 2, 3, 4], Disabled).editing("1"),
        State::new([2, 3, 4, 1], Enabled),
        FlagResult::Keep,
        |s| {
            command(s, RollDown);
        },
    );
}

#[test]
fn si_t013_roll_up() {
    verify(
        State::new([1, 2, 3, 4], Enabled).editing("1"),
        State::new([4, 1, 2, 3], Enabled),
        FlagResult::Keep,
        |s| {
            command(s, RollUp);
        },
    );
}

#[test]
fn si_t014_swap_xy() {
    verify(
        State::new([1, 2, 3, 4], Disabled).editing("1"),
        State::new([2, 1, 3, 4], Enabled),
        FlagResult::Keep,
        |s| {
            command(s, SwapXY);
        },
    );
}

#[test]
fn si_t015_rotation_cycles() {
    verify(
        State::new([1, 2, 3, 4], Enabled),
        State::new([1, 2, 3, 4], Enabled),
        FlagResult::Keep,
        |s| {
            for _ in 0..4 {
                command(s, RollDown);
            }
            for _ in 0..4 {
                command(s, RollUp);
            }
            for _ in 0..2 {
                command(s, SwapXY);
            }
        },
    );
}

#[test]
fn si_t016_clear_edit_then_digit() {
    verify(
        State::new([12, 2, 3, 4], Enabled).editing("12"),
        State::new([7, 2, 3, 4], Disabled).editing("7"),
        FlagResult::Keep,
        |s| {
            command(s, ClearX);
            digit(s, '7');
        },
    );
}

#[test]
fn si_t017_clear_unfinished_exponent() {
    verify(
        State::new([1, 2, 3, 4], Enabled).editing("1e-").float(),
        State::new([0, 2, 3, 4], Disabled).float(),
        FlagResult::Keep,
        |s| {
            command(s, ClearX);
        },
    );
}

#[test]
fn si_t018_recall_last_x_enabled() {
    verify(
        State::new([3, 5, 2, 4], Enabled),
        State::new([9, 3, 5, 2], Enabled),
        FlagResult::Keep,
        |s| {
            command(s, RecallLastX);
        },
    );
}

#[test]
fn si_t019_recall_last_x_twice() {
    verify(
        State::new([3, 5, 2, 4], Disabled),
        State::new([9, 9, 5, 2], Enabled),
        FlagResult::Keep,
        |s| {
            for _ in 0..2 {
                command(s, RecallLastX);
            }
        },
    );
}

#[test]
fn si_t020_store() {
    verify(
        State::new([12, 2, 3, 4], Disabled).editing("12"),
        State::new([12, 2, 3, 4], Enabled).register(12),
        FlagResult::Keep,
        |s| {
            command(s, Store(0));
        },
    );
}

#[test]
fn si_t021_recall_without_lift() {
    verify(
        State::new([12, 2, 3, 4], Disabled)
            .editing("12")
            .register(7),
        State::new([7, 2, 3, 4], Enabled).register(7),
        FlagResult::Keep,
        |s| {
            command(s, Recall(0));
        },
    );
}

#[test]
fn si_t022_recall_with_lift() {
    verify(
        State::new([12, 2, 3, 4], Enabled).editing("12").register(7),
        State::new([7, 12, 2, 3], Enabled).register(7),
        FlagResult::Keep,
        |s| {
            command(s, Recall(0));
        },
    );
}

#[test]
fn si_t023_push_then_add() {
    verify(
        State::new([1, 2, 3, 4], Disabled),
        State::new([8, 2, 3, 3], Enabled).history(3),
        FlagResult::Arithmetic(false, false),
        |s| {
            command(s, Push(integer(5)));
            command(s, Push(integer(3)));
            command(s, Add);
        },
    );
}

#[test]
fn si_t024_reject_push_during_edit() {
    verify(
        State::new([12, 2, 3, 4], Disabled).editing("12"),
        State::new([12, 2, 3, 4], Disabled).editing("12"),
        FlagResult::Keep,
        |s| {
            assert_eq!(
                s.apply(InputEvent::Command(Push(integer(3)))),
                Err(SessionError::Input(InputError::InputInProgress))
            );
        },
    );
}

#[test]
fn si_t025_digit_after_integer_chs() {
    verify(
        State::new([5, 2, 3, 4], Disabled).editing("5"),
        State::new([2, -5, 2, 3], Enabled).editing("2").history(5),
        FlagResult::Arithmetic(true, false),
        |s| {
            s.apply(InputEvent::Key(Key::ChangeSign)).unwrap();
            digit(s, '2');
        },
    );
}

#[test]
fn si_t026_repeated_integer_chs() {
    verify(
        State::new([5, 2, 3, 4], Enabled).editing("5"),
        State::new([5, 2, 3, 4], Enabled).history(-5),
        FlagResult::Arithmetic(true, false),
        |s| {
            s.apply(InputEvent::Key(Key::ChangeSign)).unwrap();
            s.apply(InputEvent::Key(Key::ChangeSign)).unwrap();
        },
    );
}
