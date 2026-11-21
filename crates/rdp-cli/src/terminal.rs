use crate::line::{CliError, CliSession, LineError, RunControl};
use rdp_core::{Calculator, CalculatorConfig, NumericMode, Radix, SignedMode, WordSize};
use std::io::{self, BufRead, Write};

const USAGE: &str = "Usage: rdp-calc [--base bin|oct|dec|hex] [--word-size 1..64] [--signed unsigned|ones|twos] [--float] [--register-count 1..1024] [EXPRESSION]";
struct Options {
    config: CalculatorConfig,
    expression: Option<String>,
}
fn invalid(s: &str) -> LineError {
    LineError::Syntax(CliError::InvalidArgument(s.into()))
}
fn integer<T: std::str::FromStr>(s: &str) -> Result<T, LineError> {
    if !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid(s));
    }
    s.parse().map_err(|_| invalid(s))
}
impl Options {
    fn parse(args: Vec<String>) -> Result<Self, LineError> {
        let mut config = CalculatorConfig::default();
        let mut expression = None;
        let mut seen = std::collections::HashSet::new();
        let mut args = args.into_iter();
        while let Some(arg) = args.next() {
            if !arg.starts_with("--") {
                if expression.replace(arg).is_some() {
                    return Err(invalid("expected one expression"));
                }
                continue;
            }
            if !seen.insert(arg.clone()) {
                return Err(invalid(&arg));
            }
            let mut value = || {
                args.next()
                    .ok_or_else(|| LineError::Syntax(CliError::MissingArgument(arg.clone())))
            };
            match arg.as_str() {
                "--float" => config.numeric_mode = NumericMode::FloatingPoint,
                "--base" => {
                    config.radix = match value()?.as_str() {
                        "bin" => Radix::Binary,
                        "oct" => Radix::Octal,
                        "dec" => Radix::Decimal,
                        "hex" => Radix::Hexadecimal,
                        s => return Err(invalid(s)),
                    }
                }
                "--signed" => {
                    config.word_config.signed_mode = match value()?.as_str() {
                        "unsigned" => SignedMode::Unsigned,
                        "ones" => SignedMode::OnesComplement,
                        "twos" => SignedMode::TwosComplement,
                        s => return Err(invalid(s)),
                    }
                }
                "--word-size" => {
                    let s = value()?;
                    config.word_config.size =
                        WordSize::new(integer(&s)?).map_err(|_| invalid(&s))?;
                }
                "--register-count" => config.register_count = integer(&value()?)?,
                s => return Err(invalid(s)),
            }
        }
        if config.numeric_mode == NumericMode::FloatingPoint && !seen.contains("--base") {
            config.radix = Radix::Decimal
        }
        config.validate().map_err(|e| invalid(&e.to_string()))?;
        Ok(Self { config, expression })
    }
}

pub fn run(
    args: Vec<String>,
    input: &mut impl BufRead,
    output: &mut impl Write,
    errors: &mut impl Write,
) -> Result<(), LineError> {
    if args.len() == 1 && args[0] == "--help" {
        writeln!(output, "{USAGE}")?;
    } else {
        let options = Options::parse(args)?;
        let mut session = CliSession::new(
            Calculator::with_config(options.config).map_err(LineError::Calculation)?,
        );
        if let Some(expr) = options.expression {
            session.run_line(&expr, output, false)?;
            session.finish_batch(output)?;
        } else {
            repl(&mut session, input, output, errors)?;
        }
    }
    output.flush()?;
    Ok(())
}

// Keep at most one extra byte for a possible CR before LF. Discard the
// remainder of an oversized line without allocating for it.
fn read_line(input: &mut impl BufRead, bytes: &mut Vec<u8>) -> Result<bool, LineError> {
    bytes.clear();
    let mut overflow = false;
    loop {
        let chunk = input.fill_buf()?;
        if chunk.is_empty() {
            if bytes.is_empty() {
                return Ok(false);
            }
            break;
        }
        let newline = chunk.iter().position(|&b| b == b'\n');
        let len = newline.unwrap_or(chunk.len());
        let keep = len.min((super::line::MAX_LINE_BYTES + 1) - bytes.len());
        bytes.extend_from_slice(&chunk[..keep]);
        overflow |= keep < len;
        input.consume(len + usize::from(newline.is_some()));
        if newline.is_some() {
            if bytes.last() == Some(&b'\r') {
                bytes.pop();
            }
            break;
        }
    }
    if overflow || bytes.len() > super::line::MAX_LINE_BYTES {
        return Err(LineError::Syntax(CliError::LineTooLong));
    }
    Ok(true)
}
fn repl(
    s: &mut CliSession,
    input: &mut impl BufRead,
    out: &mut impl Write,
    err: &mut impl Write,
) -> Result<(), LineError> {
    let mut buf = Vec::new();
    loop {
        let step = match read_line(input, &mut buf) {
            Ok(false) => break,
            Ok(true) => {
                let line = std::str::from_utf8(&buf)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                s.repl_line(line, out)
            }
            Err(e) => Err(e),
        };
        match step {
            Ok(RunControl::Exit) => break,
            Ok(RunControl::Continue) => {}
            Err(e @ LineError::Io(_)) => return Err(e),
            Err(e) => {
                writeln!(err, "{e}")?;
                err.flush()?;
            }
        }
        out.flush()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn write_failure_preserves_io_exit_code() {
        let mut unavailable = &mut [0u8; 0][..];
        let error = run(
            vec!["5".into()],
            &mut &b""[..],
            &mut unavailable,
            &mut Vec::new(),
        )
        .unwrap_err();
        assert!(matches!(error, LineError::Io(_)));
        assert_eq!(error.exit_code(), 3);
    }
}
