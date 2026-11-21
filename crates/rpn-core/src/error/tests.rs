use super::*;

#[test]
fn ia_c014_uses_specification_order_and_keeps_first_target_on_ties() {
    use CalcError::*;
    let errors = [
        InvariantViolation,
        AllocationFailed,
        DivisionByZero,
        DomainError,
        IntegerOutOfRange,
        FloatingPointOutOfRange,
        InvalidWordSize(0),
        InvalidRegisterCount(0),
        InvalidRegister(16),
        InvalidFlag(6),
        InvalidBitIndex(64),
        InvalidCount,
        InvalidConfiguration,
        NumericModeMismatch,
        UnsupportedOperation,
    ];
    for (index, first) in errors.iter().enumerate() {
        for second in &errors[index + 1..] {
            assert_eq!(
                first_error([Some(first.clone()), Some(second.clone())]),
                Err(first.clone())
            );
            assert_eq!(
                first_error([Some(second.clone()), Some(first.clone())]),
                Err(first.clone())
            );
        }
    }
    for first in [9, 7, 5, 3, 1] {
        assert_eq!(
            first_error([
                Some(InvalidRegister(first)),
                Some(InvalidRegister(first + 1))
            ]),
            Err(InvalidRegister(first))
        );
    }
    assert_eq!(first_error([None, None]), Ok(()));
}
