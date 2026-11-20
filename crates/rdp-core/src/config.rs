use crate::{CalcError, WordSize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Radix {
    Binary,
    Octal,
    Decimal,
    Hexadecimal,
}

impl Radix {
    pub fn base(self) -> u32 {
        match self {
            Self::Binary => 2,
            Self::Octal => 8,
            Self::Decimal => 10,
            Self::Hexadecimal => 16,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumericMode {
    Integer,
    FloatingPoint,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignedMode {
    Unsigned,
    OnesComplement,
    TwosComplement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WordConfig {
    pub size: WordSize,
    pub signed_mode: SignedMode,
}

impl WordConfig {
    pub(crate) fn range(self) -> std::ops::RangeInclusive<i128> {
        let sign = 1_i128 << (self.size.bits() - 1);
        match self.signed_mode {
            SignedMode::Unsigned => 0..=i128::from(self.size.mask()),
            SignedMode::OnesComplement => (1 - sign)..=(sign - 1),
            SignedMode::TwosComplement => -sign..=(sign - 1),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalculatorConfig {
    pub word_config: WordConfig,
    pub radix: Radix,
    pub numeric_mode: NumericMode,
    pub register_count: usize,
}

impl Default for CalculatorConfig {
    fn default() -> Self {
        Self {
            word_config: WordConfig {
                size: WordSize::W16,
                signed_mode: SignedMode::TwosComplement,
            },
            radix: Radix::Hexadecimal,
            numeric_mode: NumericMode::Integer,
            register_count: 16,
        }
    }
}

impl CalculatorConfig {
    pub fn validate(&self) -> Result<(), CalcError> {
        validate_register_count(self.register_count)?;
        if self.numeric_mode == NumericMode::FloatingPoint && self.radix != Radix::Decimal {
            return Err(CalcError::InvalidConfiguration);
        }
        Ok(())
    }
}

pub(crate) fn validate_register_count(count: usize) -> Result<(), CalcError> {
    if (1..=1024).contains(&count) {
        Ok(())
    } else {
        Err(CalcError::InvalidRegisterCount(count))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackLift {
    Enabled,
    Disabled,
}
