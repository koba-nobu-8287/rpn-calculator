use super::*;

#[test]
fn ia_c016_invalid_candidate_leaves_original_state_unchanged() {
    let mut calc = Calculator::new();
    let before = calc.snapshot();
    let result = calc.transact(|candidate| {
        candidate.stack.set_x(Number::Float(3.0));
        Ok(())
    });
    assert_eq!(result, Err(CalcError::InvariantViolation));
    assert_eq!(calc.snapshot(), before);
}

#[test]
fn invalid_actual_state_precedes_command_validation() {
    let mut calc = Calculator::new();
    calc.state.stack.set_x(Number::Float(3.0));
    let before = calc.snapshot();
    assert_eq!(
        CalculatorSnapshot::try_from(&calc),
        Err(CalcError::InvariantViolation)
    );
    assert_eq!(
        calc.execute(Command::SetFlag {
            index: 255,
            value: false
        }),
        Err(CalcError::InvariantViolation)
    );
    assert_eq!(calc.snapshot(), before);
}

#[test]
fn invalid_flags_precede_zero_divisor() {
    let mut calc = Calculator::new();
    calc.state.flags = crate::Flags::with_invalid_upper_bit();
    let before = calc.snapshot();
    assert_eq!(
        calc.execute(Command::Divide),
        Err(CalcError::InvariantViolation)
    );
    assert_eq!(calc.snapshot(), before);
}
