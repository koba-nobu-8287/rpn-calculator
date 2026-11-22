use super::*;
fn session() -> CliSession {
    CliSession::new(Calculator::new())
}
#[test]
fn syntax_checks_the_entire_line() {
    for line in [
        "1 unknown",
        "1 sf 6",
        "1 wsize 0",
        "1 maskl 65",
        "1 signed",
        "1 keys \"1",
        "1 keys \"0xff\"",
        "1 keys \"keys\"",
        "1 sto -1",
    ] {
        let mut s = session();
        let before = s.snapshot();
        assert!(
            matches!(
                s.run_line(line, &mut Vec::new(), true),
                Err(LineError::Syntax(_))
            ),
            "{line}"
        );
        assert_eq!(s.snapshot(), before);
    }
}
#[test]
fn syntax_and_runtime_failures_have_different_commit_boundaries() {
    for (line, want) in [("5 0 /", 0), ("5 0xG", 5), ("5 lj", 5), ("5 dblmul", 5)] {
        let mut s = session();
        let error = s.run_line(line, &mut Vec::new(), true).unwrap_err();
        assert!(matches!(
            (line, error),
            ("5 0 /", LineError::Calculation(CalcError::DivisionByZero))
                | (
                    "5 0xG",
                    LineError::Literal(LiteralError::InvalidDigit { .. })
                )
                | (
                    "5 lj" | "5 dblmul",
                    LineError::Calculation(CalcError::UnsupportedOperation)
                )
        ));
        assert_eq!(
            s.snapshot().calculator.stack[0]
                .as_integer()
                .unwrap()
                .bits(),
            want
        );
    }
}
#[test]
fn reserved_words_and_key_subtraction() {
    for (line, want) in [
        ("HEX ff DeC 10 ADD # ignored", 265),
        ("dec 10 - sub", 10),
        ("dec keys \"7 enter 2 -\"", 5),
        ("dec keys \"1 2\"", 12),
    ] {
        let mut s = session();
        s.run_line(line, &mut Vec::new(), true).unwrap();
        assert_eq!(
            s.snapshot().calculator.stack[0]
                .as_integer()
                .unwrap()
                .bits(),
            want
        );
    }
}
#[test]
fn batch_preflight_and_exit() {
    let mut s = session();
    let before = s.snapshot();
    assert!(matches!(
        s.run_line("5 0 / stack", &mut Vec::new(), false),
        Err(LineError::Syntax(_))
    ));
    assert_eq!(s.snapshot(), before);
    assert_eq!(
        s.run_line("exit 5", &mut Vec::new(), true).unwrap(),
        RunControl::Exit
    );
    assert_eq!(s.snapshot(), before);
}

fn fixture() -> CliSession {
    let mut s = CliSession::new(
        Calculator::with_config(CalculatorConfig {
            radix: Radix::Decimal,
            word_config: WordConfig {
                ones_zero_mode: rpn_core::OnesZeroMode::Canonical,
                size: WordSize::new(8).unwrap(),
                signed_mode: SignedMode::TwosComplement,
            },
            ..Default::default()
        })
        .unwrap(),
    );
    s.run_line(
        "9 chs clx 7 sto 0 4 3 2 5 sf 0 sf 2 sf 4 sf 5",
        &mut Vec::new(),
        true,
    )
    .unwrap();
    s
}
#[test]
fn pending_and_normal_execution_use_specification_fixture() {
    let mut s = fixture();
    let before = s.snapshot();
    let w = before.calculator.word_config.size;
    let n = |x| Number::Integer(Word::from_bits(x, w));
    assert_eq!(before.calculator.stack, [5, 2, 3, 4].map(n));
    assert_eq!(before.calculator.last_x, n(9));
    for token in ["0", "0xG"] {
        s.pending = Some(token.into());
        let e = s.submit(Some(Command::Divide)).unwrap_err();
        assert!(matches!(
            (token, e),
            ("0", LineError::Calculation(CalcError::DivisionByZero))
                | (
                    "0xG",
                    LineError::Literal(LiteralError::InvalidDigit { digit: 'G', .. })
                )
        ));
        assert_eq!(s.snapshot(), before);
        assert_eq!(s.pending.as_deref(), Some(token));
        assert!(matches!(
            s.run_line("keys \"1\"", &mut Vec::new(), true),
            Err(LineError::Input(InputError::InputInProgress))
        ));
    }
    s.pending = Some("0x123".into());
    s.submit(None).unwrap();
    let mut expected = before.clone();
    expected.calculator.stack = [35, 5, 2, 3].map(n);
    assert_eq!(s.snapshot(), expected);
    assert!(s.pending.is_none());
    let mut s = fixture();
    assert!(matches!(
        s.run_line("5 0 / 7", &mut Vec::new(), true),
        Err(LineError::Calculation(CalcError::DivisionByZero))
    ));
    expected.calculator.stack = [0, 5, 5, 2].map(n);
    assert_eq!(s.snapshot(), expected);
}

#[test]
fn bit_register_and_rotation_commands_reach_core() {
    let mut s = session();
    let mut reference = Calculator::new();
    use Command::*;
    for (token, cmd) in [
        ("maskr 3", MaskRight(3)),
        (
            "sb 5",
            EditBit {
                index: 5,
                edit: BitEdit::Set,
            },
        ),
        (
            "cb 1",
            EditBit {
                index: 1,
                edit: BitEdit::Clear,
            },
        ),
        (
            "sf 4",
            SetFlag {
                index: 4,
                value: true,
            },
        ),
        ("rlc", RotateLeftCarry),
        ("rrc", RotateRightCarry),
        ("sto 10", Store(10)),
        ("maskl 4", MaskLeft(4)),
        ("rcl 10", Recall(10)),
        ("xor", Xor),
        ("popcount", CountBits),
        ("signed ones", SetSignedMode(SignedMode::OnesComplement)),
        ("wsize 8", SetWordSize(WordSize::new(8).unwrap())),
        (
            "cf 4",
            SetFlag {
                index: 4,
                value: false,
            },
        ),
        ("rln", RotateN(RotateKind::Left)),
        ("rrn", RotateN(RotateKind::Right)),
        ("rlcn", RotateN(RotateKind::LeftCarry)),
        ("rrcn", RotateN(RotateKind::RightCarry)),
    ] {
        let expected = reference.execute(cmd);
        let result = s.run_line(token, &mut Vec::new(), true);
        match expected {
            Ok(()) => assert_eq!(result.unwrap(), RunControl::Continue),
            Err(e) => assert!(matches!(result,Err(LineError::Calculation(actual)) if actual==e)),
        }
        assert_eq!(s.snapshot().calculator, reference.snapshot(), "{token}");
    }
}

#[test]
fn queries_and_unsupported_words_are_distinct_from_unknown_syntax() {
    for token in ["stack", "status", "help", "exit", "bit? 0"] {
        let mut s = session();
        let before = s.snapshot();
        assert!(matches!(
            s.run_line(&format!("1 {token}"), &mut Vec::new(), false),
            Err(LineError::Syntax(CliError::InvalidArgument(_)))
        ));
        assert_eq!(s.snapshot(), before);
    }
    for token in [
        "lj", "dblmul", "dbldiv", "dblr", "wsize-x", "run", "label", "goto", "call", "rtn",
    ] {
        assert!(matches!(
            session().run_line(token, &mut Vec::new(), true),
            Err(LineError::Calculation(CalcError::UnsupportedOperation))
        ));
    }
    assert!(
        matches!(session().run_line("G",&mut Vec::new(),true),Err(LineError::Syntax(CliError::UnknownCommand(s))) if s=="G")
    );
}

#[test]
fn oneszero_errors_leave_the_entire_line_unexecuted() {
    let mut s = session();
    let before = s.snapshot();
    let mut output = Vec::new();
    assert!(matches!(s.run_line("1 oneszero", &mut output, true),
        Err(LineError::Syntax(CliError::MissingArgument(name))) if name == "oneszero"));
    assert_eq!(s.snapshot(), before);
    assert!(matches!(s.run_line("1 oneszero other", &mut output, true),
        Err(LineError::Syntax(CliError::InvalidArgument(value))) if value == "other"));
    assert_eq!(s.snapshot(), before);
}

#[test]
fn decimal_stack_sequences_from_reset() {
    use rpn_core::StackLift::{Disabled, Enabled};
    for (line, stack, last, lift, buffer) in [
        ("keys \"5\"", [5, 0, 0, 0], 0, Disabled, Some("5")),
        ("keys \"5 enter 3 +\"", [8, 0, 0, 0], 3, Enabled, None),
        ("keys \"5 enter enter 3 +\"", [8, 5, 0, 0], 3, Enabled, None),
        (
            "keys \"5 enter clx 3\"",
            [3, 5, 0, 0],
            0,
            Disabled,
            Some("3"),
        ),
        ("keys \"5 enter 3 + 2 *\"", [16, 0, 0, 0], 2, Enabled, None),
        ("keys \"5 enter 3 + lastx\"", [3, 8, 0, 0], 3, Enabled, None),
        (
            "keys \"1 enter 2 enter 3 enter 4 rolldown\"",
            [3, 2, 1, 4],
            0,
            Enabled,
            None,
        ),
        (
            "keys \"5 enter\" hex keys \"3 +\"",
            [8, 0, 0, 0],
            3,
            Enabled,
            None,
        ),
        (
            "keys \"5 enter 3 + clx 7\"",
            [7, 0, 0, 0],
            3,
            Disabled,
            Some("7"),
        ),
        ("5 3 +", [8, 0, 0, 0], 3, Enabled, None),
        ("5 sto 0 clx rcl 0", [5, 0, 0, 0], 0, Enabled, None),
        ("keys \"1 2 bs bs\"", [0, 0, 0, 0], 0, Disabled, Some("0")),
    ] {
        let mut s = session();
        s.run_line("dec", &mut Vec::new(), true).unwrap();
        let initial = s.snapshot().calculator;
        assert_eq!(initial.stack_lift, Disabled);
        s.run_line(line, &mut Vec::new(), true).unwrap();
        let snap = s.snapshot();
        let n = |x| Number::Integer(Word::from_bits(x, initial.word_config.size));
        assert_eq!(snap.calculator.stack, stack.map(n), "{line}");
        assert_eq!(snap.calculator.last_x, n(last), "{line}");
        assert_eq!(snap.calculator.stack_lift, lift, "{line}");
        assert_eq!(snap.input_buffer.as_deref(), buffer, "{line}");
        assert_eq!(
            snap.entry_phase,
            if buffer.is_some() {
                EntryPhase::Editing
            } else {
                EntryPhase::Idle
            }
        );
        if line.contains("sto") {
            assert_eq!(snap.calculator.registers[0], n(5));
        }
    }
}

#[test]
fn editing_and_pending_cannot_commit_together() {
    let mut s = fixture();
    s.run_line("keys \"12\"", &mut Vec::new(), true).unwrap();
    let before = s.snapshot();
    assert!(s.pending.is_none());
    assert!(matches!(
        s.run_line("3", &mut Vec::new(), true),
        Err(LineError::Input(InputError::InputInProgress))
    ));
    assert_eq!(s.snapshot(), before);
    assert!(s.pending.is_none());
    // A pre-staged token is also preserved when submit rejects the transition.
    s.pending = Some("3".into());
    assert!(matches!(
        s.submit(None),
        Err(LineError::Input(InputError::InputInProgress))
    ));
    assert_eq!(s.snapshot(), before);
    assert_eq!(s.pending.as_deref(), Some("3"));
}

#[test]
fn common_fixture_digit_edits_preserve_unrelated_state() {
    for float in [false, true] {
        let mut s = fixture();
        if float {
            s.run_line("float", &mut Vec::new(), true).unwrap();
        }
        let mut expected = s.snapshot();
        let n = |x| {
            if float {
                Number::Float(x as f64)
            } else {
                Number::Integer(Word::from_bits(x as u64, WordSize::new(8).unwrap()))
            }
        };
        expected.calculator.stack = [if float { -1 } else { 12 }, 5, 2, 3].map(n);
        expected.entry_phase = EntryPhase::Editing;
        expected.input_buffer = Some(if float { "-1" } else { "12" }.into());
        let line = if float {
            "keys \"1 2 chs bs\""
        } else {
            "keys \"1 2\""
        };
        s.run_line(line, &mut Vec::new(), true).unwrap();
        assert_eq!(s.snapshot(), expected);
    }
}

#[test]
fn signed_history_and_exponent_entry() {
    for float in [false, true] {
        let mut s = session();
        let setup = if float {
            "float 7 abs clx"
        } else {
            "dec wsize 8 7 abs clx"
        };
        s.run_line(setup, &mut Vec::new(), true).unwrap();
        s.run_line("5 chs", &mut Vec::new(), true).unwrap();
        let snap = s.snapshot();
        assert_eq!(snap.entry_phase, EntryPhase::Idle);
        assert_eq!(snap.calculator.stack_lift, StackLift::Enabled);
        if float {
            assert_eq!(snap.calculator.stack[0], Number::Float(-5.));
            assert_eq!(snap.calculator.last_x, Number::Float(7.));
            s.run_line("clx keys \"5 chs\"", &mut Vec::new(), true)
                .unwrap();
            let edited = s.snapshot();
            assert_eq!(edited.entry_phase, EntryPhase::Editing);
            assert_eq!(edited.calculator.stack[0], Number::Float(-5.0));
            assert_eq!(edited.calculator.stack_lift, StackLift::Disabled);
            assert_eq!(edited.input_buffer.as_deref(), Some("-5"));
            assert_eq!(edited.calculator.last_x, Number::Float(7.));
            s.run_line("keys \"2\"", &mut Vec::new(), true).unwrap();
            assert_eq!(s.snapshot().calculator.stack[0], Number::Float(-52.));
        } else {
            assert_eq!(snap.calculator.stack[0].as_integer().unwrap().bits(), 251);
            assert_eq!(snap.calculator.last_x.as_integer().unwrap().bits(), 5);
        }
    }
    let mut s = session();
    s.run_line(
        "float keys \"1 . 5 eex chs 2 enter\"",
        &mut Vec::new(),
        true,
    )
    .unwrap();
    let snap = s.snapshot();
    assert_eq!(snap.calculator.stack[..2], [Number::Float(0.015); 2]);
    assert_eq!(snap.calculator.stack_lift, StackLift::Disabled);
    assert_eq!(snap.entry_phase, EntryPhase::Idle);
}

#[test]
fn integer_edit_completion_and_radix_neutrality() {
    let mut s = session();
    s.run_line("dec wsize 8 keys \"5\"", &mut Vec::new(), true)
        .unwrap();
    let before = s.snapshot();
    assert!(matches!(
        s.run_line("3", &mut Vec::new(), true),
        Err(LineError::Input(InputError::InputInProgress))
    ));
    assert_eq!(s.snapshot(), before);
    s.run_line("keys \"chs\"", &mut Vec::new(), true).unwrap();
    let snap = s.snapshot();
    assert_eq!(snap.entry_phase, EntryPhase::Idle);
    assert!(snap.input_buffer.is_none());
    assert_eq!(snap.calculator.stack_lift, StackLift::Enabled);
    assert_eq!(snap.calculator.stack[0].as_integer().unwrap().bits(), 251);
    assert_eq!(snap.calculator.last_x.as_integer().unwrap().bits(), 5);
    s.run_line("keys \"2\"", &mut Vec::new(), true).unwrap();
    let stack = s.snapshot().calculator.stack;
    assert_eq!(stack[0].as_integer().unwrap().bits(), 2);
    assert_eq!(stack[1].as_integer().unwrap().bits(), 251);
    s.run_line("enter", &mut Vec::new(), true).unwrap();
    let mut expected = s.snapshot();
    assert_eq!(expected.calculator.stack_lift, StackLift::Disabled);
    expected.calculator.radix = Radix::Hexadecimal;
    s.run_line("hex", &mut Vec::new(), true).unwrap();
    assert_eq!(s.snapshot(), expected);
}
