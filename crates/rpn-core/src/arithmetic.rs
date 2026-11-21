use crate::{
    CalcError, Number, OnesZeroMode, SignedMode, Word, WordConfig, error::first_error,
    flags::FlagUpdate,
};

#[derive(Clone, Copy)]
pub(crate) enum ArithmeticOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

#[derive(Clone, Copy)]
pub(crate) enum UnaryOp {
    ChangeSign,
    Absolute,
    Sqrt,
}

pub(crate) struct EvalResult {
    pub(crate) value: Number,
    pub(crate) flags: FlagUpdate,
}

pub(crate) fn eval_binary(
    op: ArithmeticOp,
    lhs: Number,
    rhs: Number,
    config: &WordConfig,
) -> Result<EvalResult, CalcError> {
    // Precedence within each variant follows X (rhs), then Y (lhs).
    first_error([
        rhs.validate_value(config.size).err(),
        lhs.validate_value(config.size).err(),
        (lhs.mode() != rhs.mode()).then_some(CalcError::NumericModeMismatch),
    ])?;

    match (lhs, rhs) {
        (Number::Integer(y), Number::Integer(x)) => integer_binary(op, y, x, config),
        (Number::Float(y), Number::Float(x)) => {
            if x == 0.0 && matches!(op, ArithmeticOp::Divide | ArithmeticOp::Remainder) {
                return Err(CalcError::DivisionByZero);
            }
            float_result(match op {
                ArithmeticOp::Add => y + x,
                ArithmeticOp::Subtract => y - x,
                ArithmeticOp::Multiply => y * x,
                ArithmeticOp::Divide => y / x,
                ArithmeticOp::Remainder => y % x,
            })
        }
        _ => Err(CalcError::NumericModeMismatch),
    }
}

fn integer_binary(
    op: ArithmeticOp,
    lhs: Word,
    rhs: Word,
    config: &WordConfig,
) -> Result<EvalResult, CalcError> {
    let (a, b) = (lhs.bits(), rhs.bits());
    let (y, x) = (lhs.to_signed(config), rhs.to_signed(config));
    let mask = config.size.mask();
    let ones = config.signed_mode == SignedMode::OnesComplement;
    let out_of_range = |value| Some(!config.range().contains(&value));

    let (raw, carry, range) = match op {
        ArithmeticOp::Add => {
            let sum = u128::from(a) + u128::from(b);
            let carry = sum > u128::from(mask);
            let low = sum as u64 & mask;
            let corrected = low + u64::from(ones && carry);
            (corrected, Some(carry), out_of_range(y + x))
        }
        ArithmeticOp::Subtract => {
            let borrow = a < b;
            let raw = a.wrapping_sub(b).wrapping_sub(u64::from(ones && borrow));
            (raw, Some(borrow), out_of_range(y - x))
        }
        ArithmeticOp::Multiply if config.signed_mode == SignedMode::Unsigned => {
            let product = u128::from(a) * u128::from(b);
            (product as u64, None, Some(product > u128::from(mask)))
        }
        ArithmeticOp::Multiply => {
            let product = y * x;
            let raw = if ones {
                let magnitude = (product.unsigned_abs() % u128::from(mask)) as u64;
                ones_encode(
                    i128::from(magnitude),
                    lhs.is_negative(config) ^ rhs.is_negative(config),
                    mask,
                )
            } else {
                product as u64
            };
            (raw, None, out_of_range(product))
        }
        ArithmeticOp::Divide | ArithmeticOp::Remainder => {
            if x == 0 {
                return Err(CalcError::DivisionByZero);
            }
            let remainder = y % x;
            let negative_y = lhs.is_negative(config);
            if matches!(op, ArithmeticOp::Divide) {
                let quotient = y / x;
                let negative = negative_y ^ rhs.is_negative(config);
                let raw = if ones {
                    ones_encode(quotient, negative, mask)
                } else {
                    quotient as u64
                };
                (raw, Some(remainder != 0), out_of_range(quotient))
            } else {
                let raw = if ones {
                    ones_encode(remainder, negative_y, mask)
                } else {
                    remainder as u64
                };
                (raw, None, None)
            }
        }
    };
    let raw = if ones && config.ones_zero_mode == OnesZeroMode::Canonical && (raw & mask) == mask {
        0
    } else {
        raw
    };
    Ok(integer_result(raw, carry, range, config))
}

fn ones_encode(value: i128, negative: bool, mask: u64) -> u64 {
    let magnitude = value.unsigned_abs() as u64;
    if negative {
        mask ^ magnitude
    } else {
        magnitude
    }
}

pub(crate) fn eval_unary(
    op: UnaryOp,
    value: Number,
    config: &WordConfig,
) -> Result<EvalResult, CalcError> {
    value.validate_value(config.size)?;
    match value {
        Number::Float(value) => float_result(match op {
            UnaryOp::ChangeSign => -value,
            UnaryOp::Absolute => value.abs(),
            UnaryOp::Sqrt if value < 0.0 => return Err(CalcError::DomainError),
            UnaryOp::Sqrt => value.sqrt(),
        }),
        Number::Integer(word) => {
            let number = word.to_signed(config);
            let ones = config.signed_mode == SignedMode::OnesComplement;
            let (raw, carry, range) = match op {
                UnaryOp::ChangeSign => (
                    if ones { !word.bits() } else { (-number) as u64 },
                    None,
                    Some(!config.range().contains(&-number)),
                ),
                UnaryOp::Absolute => (
                    if ones && word.is_negative(config) {
                        !word.bits()
                    } else {
                        number.unsigned_abs() as u64
                    },
                    None,
                    Some(!config.range().contains(&number.abs())),
                ),
                UnaryOp::Sqrt => {
                    let nonnegative = u64::try_from(number).map_err(|_| CalcError::DomainError)?;
                    let root = nonnegative.isqrt();
                    (root, Some(root * root != nonnegative), None)
                }
            };
            Ok(integer_result(raw, carry, range, config))
        }
    }
}

fn integer_result(
    raw: u64,
    carry: Option<bool>,
    range: Option<bool>,
    config: &WordConfig,
) -> EvalResult {
    EvalResult {
        value: Number::Integer(Word::from_bits(raw, config.size)),
        flags: FlagUpdate::from_conditions(carry, range),
    }
}

fn float_result(value: f64) -> Result<EvalResult, CalcError> {
    if value.is_finite() {
        Ok(EvalResult {
            value: Number::Float(value),
            flags: FlagUpdate::keep(),
        })
    } else {
        Err(CalcError::FloatingPointOutOfRange)
    }
}
