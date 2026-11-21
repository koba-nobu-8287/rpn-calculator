use rdp_core::*;
#[cfg(test)]
mod tests;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryPhase {
    Idle,
    Editing,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryPart {
    Mantissa,
    Exponent,
}
#[derive(Debug, Clone, Copy)]
pub enum Key {
    Digit(char),
    DecimalPoint,
    Exponent,
    Backspace,
    ChangeSign,
}
pub enum InputEvent {
    Key(Key),
    Command(Command),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputError {
    InvalidDigit { digit: char, radix: Radix },
    InvalidNumber,
    IncompleteEntry,
    InputInProgress,
    InputTooLong,
    InvariantViolation,
}
impl std::fmt::Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for InputError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionError {
    Input(InputError),
    Calculation(CalcError),
}
impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input(e) => write!(f, "key input: {e}"),
            Self::Calculation(e) => write!(f, "calculation: {e}"),
        }
    }
}
impl std::error::Error for SessionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::Input(e) => e,
            Self::Calculation(e) => e,
        })
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct InputSnapshot {
    pub calculator: CalculatorSnapshot,
    pub entry_phase: EntryPhase,
    pub input_buffer: Option<String>,
    pub entry_part: EntryPart,
}
#[derive(Clone)]
pub struct HpInputSession {
    calculator: Calculator,
    buffer: String,
}
impl HpInputSession {
    pub fn new(calculator: Calculator) -> Self {
        Self {
            calculator,
            buffer: String::new(),
        }
    }
    pub fn snapshot(&self) -> InputSnapshot {
        let editing = !self.buffer.is_empty();
        InputSnapshot {
            calculator: self.calculator.snapshot(),
            entry_phase: if editing {
                EntryPhase::Editing
            } else {
                EntryPhase::Idle
            },
            entry_part: if self.buffer.contains('e') {
                EntryPart::Exponent
            } else {
                EntryPart::Mantissa
            },
            input_buffer: editing.then(|| self.buffer.clone()),
        }
    }
    pub fn reset(&mut self) {
        self.calculator.reset();
        self.buffer.clear();
    }
    pub fn test_bit(&self, index: u8) -> Result<bool, SessionError> {
        if !self.buffer.is_empty() {
            self.checked_state()?;
        }
        let value = self.calculator.test_bit(index)?;
        if self.buffer.is_empty() {
            Ok(value)
        } else {
            Err(SessionError::Input(InputError::InputInProgress))
        }
    }
    pub fn apply(&mut self, event: InputEvent) -> Result<(), SessionError> {
        let config = self.checked_state()?;
        let mut candidate = self.clone();
        candidate.apply_candidate(event, &config)?;
        if !candidate.buffer.is_empty() {
            candidate.checked_state()?;
        }
        *self = candidate;
        Ok(())
    }
    fn checked_state(&self) -> Result<CalculatorSnapshot, SessionError> {
        let config = CalculatorSnapshot::try_from(&self.calculator)?;
        if !self.buffer.is_empty() {
            let preview =
                entry_preview(&self.buffer, &config).map_err(|_| InputError::InvariantViolation)?;
            let x = config.stack[0];
            let matches = match (preview, x) {
                (Number::Float(a), Number::Float(b)) => a.to_bits() == b.to_bits(),
                _ => preview == x,
            };
            if !matches {
                return Err(InputError::InvariantViolation.into());
            }
        }
        Ok(config)
    }
    fn apply_candidate(
        &mut self,
        event: InputEvent,
        config: &CalculatorSnapshot,
    ) -> Result<(), SessionError> {
        let editing = !self.buffer.is_empty();
        let float = config.numeric_mode == NumericMode::FloatingPoint;
        let cmd = match event {
            InputEvent::Key(Key::ChangeSign) | InputEvent::Command(Command::ChangeSign)
                if !(editing && float) =>
            {
                Command::ChangeSign
            }
            InputEvent::Key(Key::Backspace) if !editing => Command::ClearX,
            InputEvent::Command(Command::ChangeSign) => return self.edit(Key::ChangeSign, config),
            InputEvent::Key(key) => return self.edit(key, config),
            InputEvent::Command(cmd) => cmd,
        };
        if editing {
            match cmd {
                Command::ClearRegisters => return Ok(self.calculator.execute(cmd)?),
                Command::ClearStack | Command::ClearX => {}
                Command::Push(_) => {
                    self.calculator.execute(cmd)?;
                    return Err(SessionError::Input(InputError::InputInProgress));
                }
                _ if incomplete(&self.buffer) => {
                    // Only operand-independent errors compete with an unfinished exponent.
                    match cmd {
                        Command::SetRadix(_)
                        | Command::SetWordSize(_)
                        | Command::SetSignedMode(_)
                        | Command::SetFlag { .. }
                        | Command::Store(_)
                        | Command::Recall(_)
                        | Command::And
                        | Command::Or
                        | Command::Xor
                        | Command::Not
                        | Command::ShiftLeft
                        | Command::ShiftRight
                        | Command::ShiftRightArithmetic
                        | Command::RotateLeft
                        | Command::RotateRight
                        | Command::RotateLeftCarry
                        | Command::RotateRightCarry
                        | Command::RotateN(_)
                        | Command::MaskLeft(_)
                        | Command::MaskRight(_)
                        | Command::EditBit { .. }
                        | Command::CountBits => self.calculator.execute(cmd)?,
                        _ => {}
                    }
                    return Err(SessionError::Input(InputError::IncompleteEntry));
                }
                _ => {} // Every complete edit already installed its exact preview in X.
            }
        }
        self.calculator.execute(cmd)?;
        self.buffer.clear();
        Ok(())
    }
    fn edit(&mut self, key: Key, config: &CalculatorSnapshot) -> Result<(), SessionError> {
        let start = self.buffer.is_empty();
        let buf = &mut self.buffer;
        let float = config.numeric_mode == NumericMode::FloatingPoint;
        match key {
            Key::DecimalPoint | Key::Exponent if !float => {
                return Err(CalcError::NumericModeMismatch.into());
            }
            Key::Digit(_) | Key::DecimalPoint | Key::Exponent if buf.len() == 256 => {
                return Err(InputError::InputTooLong.into());
            }
            Key::Digit(c) => {
                if !c.is_digit(config.radix.base()) {
                    return Err(InputError::InvalidDigit {
                        digit: c,
                        radix: config.radix,
                    }
                    .into());
                }
                buf.push(c.to_ascii_uppercase());
            }
            Key::DecimalPoint => {
                if buf.contains(['.', 'e']) {
                    return Err(InputError::InvalidNumber.into());
                }
                if start {
                    buf.push('0')
                }
                buf.push('.')
            }
            Key::Exponent => {
                if start {
                    return Err(InputError::IncompleteEntry.into());
                }
                if buf.contains('e') {
                    return Err(InputError::InvalidNumber.into());
                }
                buf.push('e')
            }
            Key::ChangeSign => {
                let at = buf.find('e').map_or(0, |i| i + 1);
                if buf[at..].starts_with('-') {
                    buf.remove(at);
                } else {
                    buf.insert(at, '-')
                }
            }
            Key::Backspace => {
                buf.pop();
                if buf.is_empty() || buf == "-" {
                    buf.clear();
                    buf.push('0')
                }
            }
        }
        let value = entry_preview(buf, config)?;
        self.calculator.update_value(if start {
            ValueUpdate::Start(value)
        } else {
            ValueUpdate::ReplaceX(value)
        })?;
        Ok(())
    }
}

fn entry_preview(buf: &str, config: &CalculatorSnapshot) -> Result<Number, SessionError> {
    if buf.len() > 256 {
        return Err(InputError::InputTooLong.into());
    }
    if config.numeric_mode == NumericMode::FloatingPoint {
        let (mantissa, exponent) = buf.split_once('e').unwrap_or((buf, ""));
        let unsigned = mantissa.strip_prefix('-').unwrap_or(mantissa);
        let (whole, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
        let exp_digits = exponent.strip_prefix('-').unwrap_or(exponent);
        if whole.is_empty()
            || ![whole, fraction, exp_digits]
                .iter()
                .all(|part| part.bytes().all(|b| b.is_ascii_digit()))
        {
            return Err(InputError::InvalidNumber.into());
        }
        let complete = buf
            .strip_suffix("e-")
            .or_else(|| buf.strip_suffix('e'))
            .unwrap_or(buf);
        let n: f64 = complete.parse().map_err(|_| InputError::InvalidNumber)?;
        if !n.is_finite() {
            return Err(CalcError::FloatingPointOutOfRange.into());
        }
        Ok(Number::Float(n))
    } else {
        let w = config.word_config;
        if !buf
            .chars()
            .all(|c| !c.is_ascii_lowercase() && c.is_digit(config.radix.base()))
        {
            return Err(InputError::InvalidNumber.into());
        }
        let n = u64::from_str_radix(buf, config.radix.base())
            .map_err(|_| CalcError::IntegerOutOfRange)?;
        let max = if config.radix == Radix::Decimal && w.signed_mode != SignedMode::Unsigned {
            w.size.mask() >> 1
        } else {
            w.size.mask()
        };
        if n > max {
            return Err(CalcError::IntegerOutOfRange.into());
        }
        Ok(Number::Integer(Word::from_bits(n, w.size)))
    }
}
fn incomplete(text: &str) -> bool {
    text.ends_with('e') || text.ends_with("e-")
}
impl From<InputError> for SessionError {
    fn from(e: InputError) -> Self {
        Self::Input(e)
    }
}
impl From<CalcError> for SessionError {
    fn from(e: CalcError) -> Self {
        Self::Calculation(e)
    }
}
