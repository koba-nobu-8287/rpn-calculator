use std::fmt::Write;

use crate::{CalcError, Flags, Number, NumericMode, Radix, WordConfig, error::first_error};

/// Formats one value with the supplied radix and display flags.
pub fn format_number(
    value: Number,
    radix: Radix,
    mode: NumericMode,
    config: &WordConfig,
    flags: &Flags,
) -> Result<String, CalcError> {
    first_error([
        value.validate(mode, config).err(),
        (mode == NumericMode::FloatingPoint && radix != Radix::Decimal)
            .then_some(CalcError::NumericModeMismatch),
    ])?;

    let mut output = String::new();
    // A raw 64-bit binary word is the longest supported representation.
    output
        .try_reserve_exact(64)
        .map_err(|_| CalcError::AllocationFailed)?;
    let written = match value {
        Number::Integer(word) => {
            let bits = usize::from(config.size.bits());
            let pad = flags.get(3)?;
            let raw = word.bits();
            match radix {
                Radix::Binary => {
                    write!(output, "{raw:0width$b}", width = if pad { bits } else { 0 })
                }
                Radix::Octal => write!(
                    output,
                    "{raw:0width$o}",
                    width = if pad { bits.div_ceil(3) } else { 0 }
                ),
                Radix::Hexadecimal => write!(
                    output,
                    "{raw:0width$X}",
                    width = if pad { bits.div_ceil(4) } else { 0 }
                ),
                Radix::Decimal if word.is_negative_zero(config) => output.write_str("-0"),
                Radix::Decimal => write!(output, "{}", word.to_signed(config)),
            }
        }
        Number::Float(number) => write!(output, "{number:?}"),
    };
    written.map_err(|_| CalcError::InvariantViolation)?;
    if output.ends_with(".0") {
        output.truncate(output.len() - 2);
    }
    Ok(output)
}
