use super::CliError;
use rdp_core::*;
use rdp_input::{InputEvent, Key};

pub(super) enum Action<'a> {
    Command(Command),
    Literal(&'a str),
    Keys(Vec<InputEvent>),
    TestBit(u8),
    Stack,
    Status,
    Help,
    Exit,
    Unsupported,
}
impl Action<'_> {
    pub(super) fn interactive_only(&self) -> bool {
        matches!(
            self,
            Self::TestBit(_) | Self::Stack | Self::Status | Self::Help | Self::Exit
        )
    }
}

fn word<'a>(rest: &mut &'a str) -> Option<&'a str> {
    let s = rest.trim_ascii_start();
    if s.is_empty() {
        return None;
    }
    let end = s.find(|c: char| c.is_ascii_whitespace()).unwrap_or(s.len());
    *rest = &s[end..];
    Some(&s[..end])
}
fn arg<'a>(rest: &mut &'a str, name: &str) -> Result<&'a str, CliError> {
    word(rest)
        .filter(|s| !s.starts_with('#'))
        .ok_or_else(|| CliError::MissingArgument(name.into()))
}
fn num<T: std::str::FromStr>(rest: &mut &str, name: &str) -> Result<T, CliError> {
    let s = arg(rest, name)?;
    if s.bytes().all(|b| b.is_ascii_digit())
        && let Ok(n) = s.parse()
    {
        return Ok(n);
    }
    Err(CliError::InvalidArgument(s.into()))
}
fn bounded(rest: &mut &str, name: &str, max: u8) -> Result<u8, CliError> {
    let n: u8 = num(rest, name)?;
    if n > max {
        Err(CliError::InvalidArgument(n.to_string()))
    } else {
        Ok(n)
    }
}
fn command(name: &str, rest: &mut &str) -> Result<Option<Command>, CliError> {
    use Command::*;
    let cmd = match name {
        "hex" => SetRadix(Radix::Hexadecimal),
        "dec" => SetRadix(Radix::Decimal),
        "oct" => SetRadix(Radix::Octal),
        "bin" => SetRadix(Radix::Binary),
        "integer" => SetNumericMode(NumericMode::Integer),
        "float" => SetNumericMode(NumericMode::FloatingPoint),
        "wsize" => {
            let n = num(rest, name)?;
            SetWordSize(WordSize::new(n).map_err(|_| CliError::InvalidArgument(n.to_string()))?)
        }
        "signed" => SetSignedMode(match arg(rest, name)?.to_ascii_lowercase().as_str() {
            "unsigned" => SignedMode::Unsigned,
            "ones" => SignedMode::OnesComplement,
            "twos" => SignedMode::TwosComplement,
            s => return Err(CliError::InvalidArgument(s.into())),
        }),
        "+" | "add" => Add,
        "sub" => Subtract,
        "*" | "mul" => Multiply,
        "/" | "div" => Divide,
        "%" | "rmd" => Remainder,
        "chs" => ChangeSign,
        "abs" => Absolute,
        "sqrt" => Sqrt,
        "and" => And,
        "or" => Or,
        "xor" => Xor,
        "not" => Not,
        "sl" => ShiftLeft,
        "sr" => ShiftRight,
        "asr" => ShiftRightArithmetic,
        "rl" => RotateLeft,
        "rr" => RotateRight,
        "rlc" => RotateLeftCarry,
        "rrc" => RotateRightCarry,
        "rln" => RotateN(RotateKind::Left),
        "rrn" => RotateN(RotateKind::Right),
        "rlcn" => RotateN(RotateKind::LeftCarry),
        "rrcn" => RotateN(RotateKind::RightCarry),
        "maskl" => MaskLeft(bounded(rest, name, 64)?),
        "maskr" => MaskRight(bounded(rest, name, 64)?),
        "sb" | "cb" => EditBit {
            index: num(rest, name)?,
            edit: if name == "sb" {
                BitEdit::Set
            } else {
                BitEdit::Clear
            },
        },
        "popcount" => CountBits,
        "enter" => Enter,
        "swap" => SwapXY,
        "rollup" => RollUp,
        "rolldown" => RollDown,
        "lastx" => RecallLastX,
        "sto" => Store(num(rest, name)?),
        "rcl" => Recall(num(rest, name)?),
        "sf" | "cf" => SetFlag {
            index: bounded(rest, name, 5)?,
            value: name == "sf",
        },
        "clx" => ClearX,
        "clearstack" => ClearStack,
        "clearregs" => ClearRegisters,
        _ => return Ok(None),
    };
    Ok(Some(cmd))
}

fn prefixed(token: &str) -> bool {
    matches!(
        token.as_bytes(),
        [
            b'0',
            b'b' | b'o' | b'd' | b'x' | b'B' | b'O' | b'D' | b'X',
            ..
        ]
    )
}
fn keys(mut rest: &str) -> Result<Vec<InputEvent>, CliError> {
    let mut result = Vec::new();
    while let Some(token) = word(&mut rest) {
        let name = token.to_ascii_lowercase();
        let event = match name.as_str() {
            "-" => InputEvent::Command(Command::Subtract),
            "." => InputEvent::Key(Key::DecimalPoint),
            "eex" => InputEvent::Key(Key::Exponent),
            "bs" => InputEvent::Key(Key::Backspace),
            "chs" => InputEvent::Key(Key::ChangeSign),
            _ if name.starts_with("digit:") => {
                let mut chars = token[6..].chars();
                let digit = chars
                    .next()
                    .filter(|_| chars.next().is_none())
                    .ok_or_else(|| CliError::InvalidArgument(token.into()))?;
                InputEvent::Key(Key::Digit(digit))
            }
            _ => {
                if let Some(cmd) = command(&name, &mut rest)? {
                    InputEvent::Command(cmd)
                } else if !prefixed(token) && token.bytes().all(|b| b.is_ascii_hexdigit()) {
                    result.extend(token.chars().map(|c| InputEvent::Key(Key::Digit(c))));
                    continue;
                } else {
                    return Err(CliError::InvalidArgument(token.into()));
                }
            }
        };
        result.push(event)
    }
    Ok(result)
}
fn literal_candidate(s: &str) -> bool {
    let body = s.strip_prefix(['+', '-']).unwrap_or(s);
    s == "-"
        || prefixed(body)
        || (!body.is_empty() && body.bytes().all(|b| b.is_ascii_hexdigit()))
        || (body.starts_with(|c: char| c.is_ascii_digit() || c == '.')
            && body
                .bytes()
                .all(|b| b.is_ascii_digit() || matches!(b, b'.' | b'e' | b'E' | b'+' | b'-')))
}
pub(super) fn parse_line(mut rest: &str) -> Result<Vec<Action<'_>>, CliError> {
    if rest.len() > 65536 {
        return Err(CliError::LineTooLong);
    }
    let mut actions = Vec::new();
    while let Some(token) = word(&mut rest) {
        if token.starts_with('#') {
            break;
        }
        let name = token.to_ascii_lowercase();
        let action = if let Some(cmd) = command(&name, &mut rest)? {
            Action::Command(cmd)
        } else {
            match name.as_str() {
                "keys" => {
                    let quoted = rest.trim_ascii_start().strip_prefix('"').ok_or_else(|| {
                        CliError::InvalidArgument("keys requires a quoted sequence".into())
                    })?;
                    let (sequence, tail) = quoted
                        .split_once('"')
                        .ok_or_else(|| CliError::InvalidArgument("unterminated keys".into()))?;
                    if !tail.is_empty() && !tail.starts_with(|c: char| c.is_ascii_whitespace()) {
                        return Err(CliError::InvalidArgument(
                            "expected space after keys".into(),
                        ));
                    }
                    rest = tail;
                    Action::Keys(keys(sequence)?)
                }
                "stack" => Action::Stack,
                "status" => Action::Status,
                "bit?" => Action::TestBit(num(&mut rest, token)?),
                "help" => Action::Help,
                "exit" => Action::Exit,
                "lj" | "dblmul" | "dbldiv" | "dblr" | "wsize-x" | "run" | "label" | "goto"
                | "call" | "rtn" => Action::Unsupported,
                _ if literal_candidate(token) => Action::Literal(token),
                _ => return Err(CliError::UnknownCommand(token.into())),
            }
        };
        actions.push(action)
    }
    Ok(actions)
}
