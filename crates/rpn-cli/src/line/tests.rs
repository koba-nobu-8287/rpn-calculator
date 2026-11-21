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
    for (line, want) in [("5 0 /", 0), ("5 0xG", 5), ("5 lj", 5)] {
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
                    "5 lj",
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
    s.run_line("9 chs clx 4 3 2 5 sto 0 sf 3", &mut Vec::new(), true)
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
