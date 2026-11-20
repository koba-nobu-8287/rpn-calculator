use crate::{CalcError, Number, config::validate_register_count};

#[derive(Clone)]
pub(crate) struct RegisterBank {
    values: Vec<Number>,
}

impl RegisterBank {
    pub(crate) fn new(count: usize, zero: Number) -> Result<Self, CalcError> {
        validate_register_count(count)?;
        let mut values = Self::allocate(count)?;
        values.resize(count, zero);
        Ok(Self { values })
    }

    fn allocate(count: usize) -> Result<Vec<Number>, CalcError> {
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| CalcError::AllocationFailed)?;
        Ok(values)
    }

    pub(crate) fn values(&self) -> &[Number] {
        &self.values
    }

    pub(crate) fn get(&self, index: usize) -> Result<Number, CalcError> {
        self.values
            .get(index)
            .copied()
            .ok_or(CalcError::InvalidRegister(index))
    }

    pub(crate) fn set(&mut self, index: usize, value: Number) -> Result<(), CalcError> {
        let slot = self
            .values
            .get_mut(index)
            .ok_or(CalcError::InvalidRegister(index))?;
        *slot = value;
        Ok(())
    }

    pub(crate) fn clear(&mut self, zero: Number) {
        self.values.fill(zero);
    }

    pub(crate) fn try_clone(&self) -> Result<Self, CalcError> {
        let mut values = self.reserve_replacement()?;
        values.extend_from_slice(&self.values);
        Ok(Self { values })
    }

    pub(crate) fn reserve_replacement(&self) -> Result<Vec<Number>, CalcError> {
        Self::allocate(self.values.len())
    }

    pub(crate) fn replace_all(&mut self, values: Vec<Number>) -> Result<(), CalcError> {
        if values.len() != self.values.len() {
            return Err(CalcError::InvariantViolation);
        }
        self.values = values;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recoverable_reservation_error_maps_to_allocation_failed() {
        assert_eq!(
            RegisterBank::allocate(usize::MAX),
            Err(CalcError::AllocationFailed)
        );
    }
}
