use crate::{CalcError, SignedMode, WordConfig};

/// An integer word width from 1 through 64 bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WordSize(u8);

impl WordSize {
    pub(crate) const W16: Self = Self(16);

    pub fn new(bits: u8) -> Result<Self, CalcError> {
        if (1..=64).contains(&bits) {
            Ok(Self(bits))
        } else {
            Err(CalcError::InvalidWordSize(bits))
        }
    }

    pub fn bits(self) -> u8 {
        self.0
    }

    pub fn mask(self) -> u64 {
        u64::MAX >> (64 - self.0)
    }

    pub(crate) fn sign_bit(self) -> u64 {
        1 << (self.0 - 1)
    }
}

/// Raw word bits. Width and signed interpretation come from the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Word(u64);

impl Word {
    /// Retains only the low bits of the given width.
    pub fn from_bits(bits: u64, size: WordSize) -> Self {
        Self(bits & size.mask())
    }

    pub fn bits(self) -> u64 {
        self.0
    }

    pub fn masked(self, size: WordSize) -> Self {
        Self::from_bits(self.0, size)
    }

    pub fn is_negative(self, config: &WordConfig) -> bool {
        config.signed_mode != SignedMode::Unsigned && self.0 & config.size.sign_bit() != 0
    }

    pub fn is_negative_zero(self, config: &WordConfig) -> bool {
        config.signed_mode == SignedMode::OnesComplement
            && self.masked(config.size).bits() == config.size.mask()
    }

    pub fn to_signed(self, config: &WordConfig) -> i128 {
        let raw = i128::from(self.masked(config.size).bits());
        if !self.is_negative(config) {
            return raw;
        }
        match config.signed_mode {
            SignedMode::OnesComplement => raw - i128::from(config.size.mask()),
            SignedMode::TwosComplement => raw - (1_i128 << config.size.bits()),
            SignedMode::Unsigned => raw,
        }
    }

    pub(crate) fn from_integer(value: i128, config: &WordConfig) -> Result<Self, CalcError> {
        if !config.range().contains(&value) {
            return Err(CalcError::IntegerOutOfRange);
        }
        let bits = if value < 0 && config.signed_mode == SignedMode::OnesComplement {
            (i128::from(config.size.mask()) + value) as u64
        } else {
            value as u64
        };
        Ok(Self::from_bits(bits, config.size))
    }

    pub(crate) fn validate(self, size: WordSize) -> Result<(), CalcError> {
        if self.masked(size) == self {
            Ok(())
        } else {
            Err(CalcError::IntegerOutOfRange)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::OnesZeroMode;

    #[test]
    fn test_008_exact_integer_bounds_for_every_width_and_sign_mode() {
        for width in 1..=64 {
            let sign = 1_i128 << (width - 1);
            for (signed_mode, min, max) in [
                (SignedMode::Unsigned, 0, 2 * sign - 1),
                (SignedMode::TwosComplement, -sign, sign - 1),
                (SignedMode::OnesComplement, 1 - sign, sign - 1),
            ] {
                let config = WordConfig {
                    size: WordSize::new(width).unwrap(),
                    signed_mode,
                    ones_zero_mode: OnesZeroMode::Canonical,
                };
                for value in [min, 0, max] {
                    let word = Word::from_integer(value, &config).unwrap();
                    assert_eq!(word.to_signed(&config), value);
                    if value == 0 {
                        assert_eq!(word.bits(), 0);
                    }
                }
                for value in [min - 1, max + 1] {
                    assert_eq!(
                        Word::from_integer(value, &config),
                        Err(CalcError::IntegerOutOfRange)
                    );
                }
            }
        }
    }
}
