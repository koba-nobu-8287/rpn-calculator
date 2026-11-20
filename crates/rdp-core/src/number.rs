use crate::{CalcError, NumericMode, Word, WordConfig, WordSize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
    Integer(Word),
    Float(f64),
}

impl Number {
    pub fn as_integer(&self) -> Option<Word> {
        match self {
            Self::Integer(word) => Some(*word),
            Self::Float(_) => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            Self::Float(value) => Some(*value),
            Self::Integer(_) => None,
        }
    }

    pub(crate) fn mode(self) -> NumericMode {
        match self {
            Self::Integer(_) => NumericMode::Integer,
            Self::Float(_) => NumericMode::FloatingPoint,
        }
    }

    pub(crate) fn zero(mode: NumericMode, size: WordSize) -> Self {
        match mode {
            NumericMode::Integer => Self::Integer(Word::from_bits(0, size)),
            NumericMode::FloatingPoint => Self::Float(0.0),
        }
    }

    pub(crate) fn validate_value(self, size: WordSize) -> Result<(), CalcError> {
        match self {
            Self::Integer(word) => word.validate(size),
            Self::Float(value) if value.is_finite() => Ok(()),
            Self::Float(_) => Err(CalcError::FloatingPointOutOfRange),
        }
    }

    pub(crate) fn validate(&self, mode: NumericMode, config: &WordConfig) -> Result<(), CalcError> {
        self.validate_value(config.size)?;
        if self.mode() == mode {
            Ok(())
        } else {
            Err(CalcError::NumericModeMismatch)
        }
    }
}
