use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CalcError {
    InvariantViolation,
    AllocationFailed,
    DivisionByZero,
    DomainError,
    IntegerOutOfRange,
    FloatingPointOutOfRange,
    InvalidWordSize(u8),
    InvalidRegisterCount(usize),
    InvalidRegister(usize),
    InvalidFlag(u8),
    InvalidBitIndex(u8),
    InvalidCount,
    InvalidConfiguration,
    NumericModeMismatch,
    UnsupportedOperation,
}

impl CalcError {
    fn priority(&self) -> u8 {
        match self {
            Self::InvariantViolation => 0,
            Self::AllocationFailed => 1,
            Self::DivisionByZero => 2,
            Self::DomainError => 3,
            Self::IntegerOutOfRange => 4,
            Self::FloatingPointOutOfRange => 5,
            Self::InvalidWordSize(_) => 6,
            Self::InvalidRegisterCount(_) => 7,
            Self::InvalidRegister(_) => 8,
            Self::InvalidFlag(_) => 9,
            Self::InvalidBitIndex(_) => 10,
            Self::InvalidCount => 11,
            Self::InvalidConfiguration => 12,
            Self::NumericModeMismatch => 13,
            Self::UnsupportedOperation => 14,
        }
    }
}

pub(crate) fn first_error(
    candidates: impl IntoIterator<Item = Option<CalcError>>,
) -> Result<(), CalcError> {
    match candidates
        .into_iter()
        .flatten()
        .min_by_key(CalcError::priority)
    {
        Some(error) => Err(error),
        None => Ok(()),
    }
}

impl fmt::Display for CalcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvariantViolation => "calculator invariant violated",
            Self::AllocationFailed => "allocation failed",
            Self::DivisionByZero => "division by zero",
            Self::DomainError => "value outside operation domain",
            Self::IntegerOutOfRange => "integer out of range",
            Self::FloatingPointOutOfRange => "floating-point value out of range",
            Self::InvalidWordSize(bits) => return write!(f, "invalid word size: {bits}"),
            Self::InvalidRegisterCount(count) => {
                return write!(f, "invalid register count: {count}");
            }
            Self::InvalidRegister(index) => return write!(f, "invalid register: {index}"),
            Self::InvalidFlag(index) => return write!(f, "invalid flag: {index}"),
            Self::InvalidBitIndex(index) => return write!(f, "invalid bit index: {index}"),
            Self::InvalidCount => "invalid count",
            Self::InvalidConfiguration => "invalid configuration",
            Self::NumericModeMismatch => "numeric mode mismatch",
            Self::UnsupportedOperation => "unsupported operation",
        };
        f.write_str(message)
    }
}

impl std::error::Error for CalcError {}

#[cfg(test)]
mod tests;
