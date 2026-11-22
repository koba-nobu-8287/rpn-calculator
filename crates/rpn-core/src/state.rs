use crate::{
    CalcError, CalculatorConfig, Flags, Number, NumericMode, Radix, SignedMode, StackLift, Word,
    WordConfig, register::RegisterBank, stack::RpnStack,
};

/// An owned copy of the current calculator state in X, Y, Z, T order.
#[derive(Debug, Clone, PartialEq)]
pub struct CalculatorSnapshot {
    pub stack: [Number; 4],
    pub last_x: Number,
    pub registers: Vec<Number>,
    pub flags: Flags,
    pub word_config: WordConfig,
    pub radix: Radix,
    pub numeric_mode: NumericMode,
    pub stack_lift: StackLift,
}

#[derive(Clone)]
pub(crate) struct CalculatorState {
    pub(crate) stack: RpnStack,
    pub(crate) registers: RegisterBank,
    pub(crate) flags: Flags,
    pub(crate) word_config: WordConfig,
    pub(crate) radix: Radix,
    pub(crate) numeric_mode: NumericMode,
    pub(crate) stack_lift: StackLift,
}

impl CalculatorState {
    pub(crate) fn new(config: &CalculatorConfig) -> Result<Self, CalcError> {
        config.validate()?;
        let zero = Number::zero(config.numeric_mode, config.word_config.size);
        Ok(Self {
            stack: RpnStack::new(zero),
            registers: RegisterBank::new(config.register_count, zero)?,
            flags: Flags::default(),
            word_config: config.word_config,
            radix: config.radix,
            numeric_mode: config.numeric_mode,
            stack_lift: StackLift::Disabled,
        })
    }

    pub(crate) fn validate(&self, register_count: usize) -> Result<(), CalcError> {
        if self.registers.values().len() != register_count
            || !self.flags.is_valid()
            || (self.numeric_mode == NumericMode::FloatingPoint && self.radix != Radix::Decimal)
        {
            return Err(CalcError::InvariantViolation);
        }
        for number in self
            .stack
            .values()
            .into_iter()
            .chain([self.stack.last_x()])
            .chain(self.registers.values().iter().copied())
        {
            number
                .validate(self.numeric_mode, &self.word_config)
                .map_err(|_| CalcError::InvariantViolation)?;
        }
        Ok(())
    }

    pub(crate) fn try_clone(&self) -> Result<Self, CalcError> {
        Ok(Self {
            stack: self.stack,
            registers: self.registers.try_clone()?,
            flags: self.flags,
            word_config: self.word_config,
            radix: self.radix,
            numeric_mode: self.numeric_mode,
            stack_lift: self.stack_lift,
        })
    }

    pub(crate) fn snapshot(&self) -> CalculatorSnapshot {
        CalculatorSnapshot {
            stack: self.stack.values(),
            last_x: self.stack.last_x(),
            registers: self.registers.values().to_vec(),
            flags: self.flags,
            word_config: self.word_config,
            radix: self.radix,
            numeric_mode: self.numeric_mode,
            stack_lift: self.stack_lift,
        }
    }

    pub(crate) fn zero(&self) -> Number {
        Number::zero(self.numeric_mode, self.word_config.size)
    }

    pub(crate) fn reconfigure_word(&mut self, next: WordConfig) -> Result<(), CalcError> {
        if self.numeric_mode != NumericMode::Integer {
            return Err(CalcError::NumericModeMismatch);
        }
        if self.word_config.size != next.size {
            self.map_values(|number| {
                let word = number.as_integer().ok_or(CalcError::InvariantViolation)?;
                Ok(Number::Integer(word.masked(next.size)))
            })?;
        }
        self.word_config = next;
        Ok(())
    }

    pub(crate) fn convert_mode(&mut self, next: NumericMode) -> Result<(), CalcError> {
        if self.numeric_mode == next {
            return Ok(());
        }
        let config = self.word_config;
        self.map_values(|number| match number {
            Number::Integer(word) => {
                if word.is_negative_zero(&config) {
                    return Ok(Number::Float(-0.0));
                }
                let integer = word.to_signed(&config);
                let float = integer as f64;
                // Every supported integer, including 2^64 after rounding, fits in i128.
                if float as i128 != integer {
                    return Err(CalcError::IntegerOutOfRange);
                }
                Ok(Number::Float(float))
            }
            Number::Float(value) => {
                if !value.is_finite() {
                    return Err(CalcError::FloatingPointOutOfRange);
                }
                if value.fract() != 0.0 || value.abs() > (1_u128 << 64) as f64 {
                    return Err(CalcError::IntegerOutOfRange);
                }
                if value == 0.0
                    && value.is_sign_negative()
                    && config.signed_mode == SignedMode::OnesComplement
                {
                    return Ok(Number::Integer(Word::from_bits(
                        config.size.mask(),
                        config.size,
                    )));
                }
                // The bound above ensures this cast cannot saturate; from_integer
                // checks the exact signed-mode limits, even when f64 rounds them.
                Ok(Number::Integer(Word::from_integer(value as i128, &config)?))
            }
        })?;
        self.numeric_mode = next;
        self.radix = Radix::Decimal;
        Ok(())
    }

    fn map_values(
        &mut self,
        mut convert: impl FnMut(Number) -> Result<Number, CalcError>,
    ) -> Result<(), CalcError> {
        // Reserve before checking values so a recoverable allocation error has priority.
        let mut registers = self.registers.reserve_replacement()?;
        let stack = self.stack.try_map(&mut convert)?;
        for &value in self.registers.values() {
            registers.push(convert(value)?);
        }
        self.registers.replace_all(registers)?;
        self.stack = stack;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn conversion_visits_stack_history_then_registers_in_order() {
        let mut state = CalculatorState::new(&CalculatorConfig {
            register_count: 2,
            ..Default::default()
        })
        .unwrap();
        let n = |x| Number::Integer(Word::from_bits(x, state.word_config.size));
        for value in [4, 3, 2, 1] {
            state.stack.lift();
            state.stack.set_x(n(value));
        }
        state.stack.set_last_x(n(5));
        state.registers.set(0, n(6)).unwrap();
        state.registers.set(1, n(7)).unwrap();
        let expected = [1, 2, 3, 4, 5, 6, 7].map(n);
        let mut seen = Vec::new();
        state
            .map_values(|v| {
                seen.push(v);
                Ok(v)
            })
            .unwrap();
        assert_eq!(seen, expected);
    }
}
