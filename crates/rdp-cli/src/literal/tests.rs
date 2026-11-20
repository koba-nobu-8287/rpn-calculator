use super::*;
use rdp_core::{SignedMode, SignedMode::*, WordSize};
fn context(bits: u8, signed_mode: SignedMode) -> ParseContext {
    ParseContext {
        radix: Radix::Decimal,
        mode: NumericMode::Integer,
        word_config: WordConfig {
            size: WordSize::new(bits).unwrap(),
            signed_mode,
        },
    }
}
fn raw(s: &str, c: ParseContext) -> Result<u64, LiteralError> {
    parse_literal(s, c).map(|v| v.as_integer().unwrap().bits())
}
// IA-L001..011 and L016
#[test]
fn integer_vectors() {
    for (mode, cases) in [
        (
            Unsigned,
            vec![("0", 0), ("255", 255), ("256", 0), ("-1", 255), ("-256", 0)],
        ),
        (
            TwosComplement,
            vec![("127", 127), ("128", 128), ("-128", 128), ("-129", 127)],
        ),
        (
            OnesComplement,
            vec![("127", 127), ("128", 128), ("-127", 128), ("-128", 127)],
        ),
    ] {
        let c = context(8, mode);
        for (s, n) in cases.into_iter().chain([
            ("0x12", 18),
            ("0x123", 35),
            ("0x1FF", 255),
            ("0x100", 0),
            ("-", 0),
            ("-0", if mode == OnesComplement { 255 } else { 0 }),
            ("-123", if mode == OnesComplement { 132 } else { 133 }),
        ]) {
            assert_eq!(raw(s, c), Ok(n), "{s} {mode:?}");
        }
        for (s, n) in [
            ("0", 0),
            ("1", 1),
            ("2", 0),
            ("-1", u64::from(mode != OnesComplement)),
        ] {
            assert_eq!(raw(s, context(1, mode)), Ok(n));
        }
    }
    assert_eq!(
        raw(
            " -0D123\t",
            ParseContext {
                radix: Radix::Hexadecimal,
                ..context(8, TwosComplement)
            }
        ),
        Ok(133)
    );
}
#[test]
fn long_inputs() {
    let c = context(64, Unsigned);
    for (s, n) in [
        ("18446744073709551615", u64::MAX),
        ("18446744073709551616", 0),
    ] {
        assert_eq!(raw(s, c), Ok(n));
    }
    assert_eq!(raw(&format!("0X{}", "F".repeat(254)), c), Ok(u64::MAX));
    for tail in ["F", "G"] {
        assert_eq!(
            raw(&format!("0x{}{tail}", "F".repeat(254)), c),
            Err(LiteralError::InputTooLong)
        );
    }
}
// IA-L012..015
#[test]
fn invalid_tokens() {
    let c = context(8, Unsigned);
    for s in ["", "+", "--1", "1+2", "0x", "0d-1", "1 2"] {
        assert_eq!(raw(s, c), Err(LiteralError::InvalidNumber));
    }
    for (s, digit, radix) in [
        ("0xGH", 'G', Radix::Hexadecimal),
        ("0b2", '2', Radix::Binary),
        ("0o8", '8', Radix::Octal),
        ("1_0", '_', Radix::Decimal),
        ("\u{ff11}", '\u{ff11}', Radix::Decimal),
    ] {
        assert_eq!(raw(s, c), Err(LiteralError::InvalidDigit { digit, radix }));
    }
}
#[test]
fn float_grammar_and_bits() {
    let c = ParseContext {
        mode: NumericMode::FloatingPoint,
        ..context(8, Unsigned)
    };
    for (s, n) in [
        ("-", 0.0),
        ("-0", -0.0),
        ("+.5", 0.5),
        ("1.E+2", 100.0),
        ("-1e-999", -0.0),
        ("5e-324", f64::from_bits(1)),
    ] {
        assert_eq!(
            parse_literal(s, c).unwrap().as_float().unwrap().to_bits(),
            n.to_bits()
        );
    }
    for s in ["1e-", "NaN", "-Inf", "0x1", ".", "1_0", "+", "1e2e3"] {
        assert_eq!(parse_literal(s, c), Err(LiteralError::InvalidNumber));
    }
    assert_eq!(
        parse_literal("1e999", c),
        Err(LiteralError::FloatingPointOutOfRange)
    );
}

// Low 64 bits computed independently with arbitrary-precision integer arithmetic.
#[test]
fn long_decimal_at_every_width() {
    let text = "1234567890".repeat(25);
    let low = 0xaccff196ce3f0ad2_u64;
    for bits in 1..=64 {
        let mask = u64::MAX >> (64 - bits);
        for mode in [Unsigned, TwosComplement, OnesComplement] {
            let c = context(bits, mode);
            assert_eq!(raw(&text, c), Ok(low & mask));
            let negative = if mode == OnesComplement {
                !low
            } else {
                low.wrapping_neg()
            };
            assert_eq!(raw(&format!("-{text}"), c), Ok(negative & mask));
        }
    }
}
