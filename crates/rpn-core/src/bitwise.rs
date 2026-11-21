use crate::{
    CalcError, SignedMode, Word, WordConfig, WordSize, error::first_error, flags::FlagUpdate,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotateKind {
    Left,
    Right,
    LeftCarry,
    RightCarry,
}

impl RotateKind {
    pub(crate) fn max_count(self, size: WordSize) -> u8 {
        size.bits() + u8::from(self.through_carry())
    }

    fn through_carry(self) -> bool {
        matches!(self, Self::LeftCarry | Self::RightCarry)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitEdit {
    Set,
    Clear,
}

pub(crate) enum LogicOp {
    And,
    Or,
    Xor,
    Not,
}
pub(crate) enum ShiftOp {
    Left,
    RightLogical,
    RightArithmetic,
}

pub(crate) struct BitwiseResult {
    pub(crate) value: Word,
    pub(crate) flags: FlagUpdate,
}

fn result(raw: u64, size: WordSize, carry: Option<bool>) -> BitwiseResult {
    BitwiseResult {
        value: Word::from_bits(raw, size),
        flags: FlagUpdate::from_conditions(carry, None),
    }
}

pub(crate) fn logic(
    op: LogicOp,
    lhs: Word,
    rhs: Option<Word>,
    size: WordSize,
) -> Result<BitwiseResult, CalcError> {
    first_error([
        rhs.and_then(|word| word.validate(size).err()),
        lhs.validate(size).err(),
    ])?;
    let a = lhs.bits();
    let raw = match (op, rhs.map(Word::bits)) {
        (LogicOp::Not, None) => !a,
        (LogicOp::And, Some(b)) => a & b,
        (LogicOp::Or, Some(b)) => a | b,
        (LogicOp::Xor, Some(b)) => a ^ b,
        _ => return Err(CalcError::InvalidConfiguration),
    };
    Ok(result(raw, size, None))
}

pub(crate) fn shift(
    op: ShiftOp,
    value: Word,
    config: &WordConfig,
) -> Result<BitwiseResult, CalcError> {
    value.validate(config.size)?;
    let raw = value.bits();
    let sign_mask = config.size.sign_bit();
    let (shifted, carry) = match op {
        ShiftOp::Left => (raw << 1, raw & sign_mask != 0),
        ShiftOp::RightLogical => (raw >> 1, raw & 1 != 0),
        ShiftOp::RightArithmetic => {
            let sign = if config.signed_mode == SignedMode::Unsigned {
                0
            } else {
                raw & sign_mask
            };
            ((raw >> 1) | sign, raw & 1 != 0)
        }
    };
    Ok(result(shifted, config.size, Some(carry)))
}

pub(crate) fn rotate(
    kind: RotateKind,
    value: Word,
    count: u8,
    size: WordSize,
    carry_in: bool,
) -> Result<BitwiseResult, CalcError> {
    value.validate(size)?;
    if count > kind.max_count(size) {
        return Err(CalcError::InvalidCount);
    }
    let mut raw = value.bits();
    let mut carry = carry_in;
    let through_carry = kind.through_carry();
    for _ in 0..count {
        let (shifted, outgoing, insertion) = match kind {
            RotateKind::Left | RotateKind::LeftCarry => (raw << 1, raw & size.sign_bit() != 0, 1),
            RotateKind::Right | RotateKind::RightCarry => (raw >> 1, raw & 1 != 0, size.sign_bit()),
        };
        let incoming = if through_carry { carry } else { outgoing };
        raw = (shifted | if incoming { insertion } else { 0 }) & size.mask();
        carry = outgoing;
    }
    Ok(result(raw, size, (count != 0).then_some(carry)))
}

pub(crate) fn validate_count(count: u8, size: WordSize) -> Result<(), CalcError> {
    if count <= size.bits() {
        Ok(())
    } else {
        Err(CalcError::InvalidCount)
    }
}

pub(crate) fn validate_index(index: u8, size: WordSize) -> Result<(), CalcError> {
    if index < size.bits() {
        Ok(())
    } else {
        Err(CalcError::InvalidBitIndex(index))
    }
}

pub(crate) fn mask_left(count: u8, size: WordSize) -> Result<BitwiseResult, CalcError> {
    validate_count(count, size)?;
    let raw = if count == size.bits() {
        size.mask()
    } else {
        size.mask() ^ (size.mask() >> count)
    };
    Ok(result(raw, size, None))
}

pub(crate) fn mask_right(count: u8, size: WordSize) -> Result<BitwiseResult, CalcError> {
    validate_count(count, size)?;
    let raw = if count == 0 {
        0
    } else {
        u64::MAX >> (64 - count)
    };
    Ok(result(raw, size, None))
}

pub(crate) fn edit_bit(
    value: Word,
    index: u8,
    edit: BitEdit,
    size: WordSize,
) -> Result<BitwiseResult, CalcError> {
    value.validate(size)?;
    validate_index(index, size)?;
    let bit = 1 << index;
    let raw = match edit {
        BitEdit::Set => value.bits() | bit,
        BitEdit::Clear => value.bits() & !bit,
    };
    Ok(result(raw, size, None))
}

pub(crate) fn test_bit(value: Word, index: u8, size: WordSize) -> Result<bool, CalcError> {
    value.validate(size)?;
    validate_index(index, size)?;
    Ok(value.bits() & (1 << index) != 0)
}

pub(crate) fn count_bits(value: Word, size: WordSize) -> BitwiseResult {
    result(u64::from(value.bits().count_ones()), size, None)
}
