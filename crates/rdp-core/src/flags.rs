use crate::CalcError;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Flags {
    bits: u8,
}

#[derive(Clone, Copy)]
enum FlagAction {
    Keep,
    Clear,
    Set,
}

#[derive(Clone, Copy)]
pub(crate) struct FlagUpdate {
    carry: FlagAction,
    out_of_range: FlagAction,
}

impl FlagUpdate {
    pub(crate) fn keep() -> Self {
        Self::from_conditions(None, None)
    }

    pub(crate) fn from_conditions(carry: Option<bool>, out_of_range: Option<bool>) -> Self {
        fn action(condition: Option<bool>) -> FlagAction {
            match condition {
                None => FlagAction::Keep,
                Some(false) => FlagAction::Clear,
                Some(true) => FlagAction::Set,
            }
        }
        Self {
            carry: action(carry),
            out_of_range: action(out_of_range),
        }
    }
}

impl Flags {
    pub fn get(&self, index: u8) -> Result<bool, CalcError> {
        Ok(self.bits & Self::mask(index)? != 0)
    }

    pub fn carry(&self) -> bool {
        self.bits & (1 << 4) != 0
    }

    pub fn out_of_range(&self) -> bool {
        self.bits & (1 << 5) != 0
    }

    fn mask(index: u8) -> Result<u8, CalcError> {
        if index < 6 {
            Ok(1 << index)
        } else {
            Err(CalcError::InvalidFlag(index))
        }
    }

    pub(crate) fn set(&mut self, index: u8, value: bool) -> Result<(), CalcError> {
        let mask = Self::mask(index)?;
        self.bits = (self.bits & !mask) | if value { mask } else { 0 };
        Ok(())
    }

    pub(crate) fn apply(&mut self, update: FlagUpdate) {
        for (mask, action) in [(1 << 4, update.carry), (1 << 5, update.out_of_range)] {
            match action {
                FlagAction::Keep => {}
                FlagAction::Clear => self.bits &= !mask,
                FlagAction::Set => self.bits |= mask,
            }
        }
    }

    pub(crate) fn is_valid(&self) -> bool {
        self.bits & 0xc0 == 0
    }
}
