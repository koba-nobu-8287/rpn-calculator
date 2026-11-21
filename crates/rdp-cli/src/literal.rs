use rdp_core::{Number, NumericMode, Radix, WordConfig};
#[derive(Clone, Copy)]
pub(crate) struct ParseContext {
    pub(crate) radix: Radix,
    pub(crate) mode: NumericMode,
    pub(crate) word_config: WordConfig,
}
#[derive(Debug, PartialEq, Eq)]
pub enum LiteralError {
    InputTooLong,
    InvalidDigit { digit: char, radix: Radix },
    InvalidNumber,
    FloatingPointOutOfRange,
}

pub(crate) fn parse_literal(text: &str, ctx: ParseContext) -> Result<Number, LiteralError> {
    let token = text.trim_ascii();
    if token.len() > 256 {
        return Err(LiteralError::InputTooLong);
    }
    if token.is_empty() || token.bytes().any(|b| b.is_ascii_whitespace()) {
        return Err(LiteralError::InvalidNumber);
    }
    let token = if token == "-" { "0" } else { token };
    if ctx.mode == NumericMode::FloatingPoint {
        // Rust's decimal grammar also accepts named infinities and NaN; exclude those spellings.
        if !token
            .bytes()
            .all(|b| b.is_ascii_digit() || matches!(b, b'e' | b'E' | b'.' | b'+' | b'-'))
        {
            return Err(LiteralError::InvalidNumber);
        }
        let n = token
            .parse::<f64>()
            .map_err(|_| LiteralError::InvalidNumber)?;
        return if n.is_finite() {
            Ok(Number::Float(n))
        } else {
            Err(LiteralError::FloatingPointOutOfRange)
        };
    }
    let negative = token.starts_with('-');
    let magnitude = token.strip_prefix(['+', '-']).unwrap_or(token);
    let prefix = match magnitude.as_bytes() {
        [b'0', letter, ..] => match letter.to_ascii_lowercase() {
            b'b' => Some(Radix::Binary),
            b'o' => Some(Radix::Octal),
            b'd' => Some(Radix::Decimal),
            b'x' => Some(Radix::Hexadecimal),
            _ => None,
        },
        _ => None,
    };
    let (radix, digits) = match prefix {
        Some(r) => (r, &magnitude[2..]),
        None => (ctx.radix, magnitude),
    };
    if digits.is_empty() || digits.contains(['+', '-']) {
        return Err(LiteralError::InvalidNumber);
    }
    let size = ctx.word_config.size;
    let mask = u128::from(size.mask());
    let mut magnitude = 0_u128;
    for digit in digits.chars() {
        let value = digit
            .to_digit(radix.base())
            .ok_or(LiteralError::InvalidDigit { digit, radix })?;
        magnitude = (magnitude * u128::from(radix.base()) + u128::from(value)) & mask;
    }
    let raw = magnitude as u64;
    let bits = if !negative {
        raw
    } else if ctx.word_config.signed_mode == rdp_core::SignedMode::OnesComplement {
        !raw
    } else {
        raw.wrapping_neg()
    };
    Ok(Number::Integer(rdp_core::Word::from_bits(bits, size)))
}
#[cfg(test)]
mod tests;
