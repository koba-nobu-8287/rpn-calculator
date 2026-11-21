use crate::{LiteralEntryError, LiteralError, enter_literal};
use rdp_core::*;
use rdp_input::*;
use std::io::{self, Write};
#[derive(Debug)]
pub enum LineError {
    Syntax(CliError),
    Literal(LiteralError),
    Input(InputError),
    Calculation(CalcError),
    Io(io::Error),
}
#[derive(Debug)]
pub enum CliError {
    UnknownCommand(String),
    MissingArgument(String),
    InvalidArgument(String),
    LineTooLong,
}
#[derive(Debug, PartialEq, Eq)]
pub enum RunControl {
    Continue,
    Exit,
}
pub struct CliSession {
    hp: HpInputSession,
    pending: Option<String>,
}
impl CliSession {
    pub fn new(calculator: Calculator) -> Self {
        Self {
            hp: HpInputSession::new(calculator),
            pending: None,
        }
    }
    pub fn snapshot(&self) -> InputSnapshot {
        self.hp.snapshot()
    }

    fn submit(&mut self, next: Option<Command>) -> Result<(), LineError> {
        if let Some(token) = &self.pending {
            if self.hp.snapshot().entry_phase == EntryPhase::Editing {
                return Err(LineError::Input(InputError::InputInProgress));
            }
            let mut candidate = self.hp.clone();
            enter_literal(&mut candidate, token)?;
            if let Some(cmd) = next {
                candidate.apply(InputEvent::Command(cmd))?;
            }
            self.hp = candidate;
            self.pending = None;
        } else if let Some(cmd) = next {
            self.hp.apply(InputEvent::Command(cmd))?;
        }
        Ok(())
    }
    /// Checks the whole line before running its actions. Queries write to output.
    pub fn run_line(
        &mut self,
        line: &str,
        output: &mut impl Write,
        interactive: bool,
    ) -> Result<RunControl, LineError> {
        use parser::Action;
        let actions = parser::parse_line(line).map_err(LineError::Syntax)?;
        if !interactive && actions.iter().any(Action::interactive_only) {
            return Err(LineError::Syntax(CliError::InvalidArgument(
                "interactive command in batch".into(),
            )));
        }
        for action in actions {
            match action {
                Action::Literal(token) => enter_literal(&mut self.hp, token)?,
                Action::Command(cmd) => self.submit(Some(cmd))?,
                Action::Keys(events) => {
                    if self.pending.is_some() {
                        return Err(LineError::Input(InputError::InputInProgress));
                    }
                    for event in events {
                        self.hp.apply(event)?;
                    }
                }
                Action::Unsupported => {
                    return Err(LineError::Calculation(CalcError::UnsupportedOperation));
                }
                Action::Exit => return Ok(RunControl::Exit),
                Action::TestBit(index) => {
                    writeln!(output, "{}", u8::from(self.hp.test_bit(index)?))?
                }
                Action::Stack => {
                    let s = self.snapshot().calculator;
                    for (label, value) in ["T", "Z", "Y", "X"]
                        .into_iter()
                        .zip(s.stack.into_iter().rev())
                    {
                        let formatted =
                            format_number(value, s.radix, s.numeric_mode, &s.word_config, &s.flags)
                                .map_err(LineError::Calculation)?;
                        writeln!(output, "{label}: {formatted}")?;
                    }
                }
                Action::Status => {
                    let snapshot = self.snapshot();
                    let s = snapshot.calculator;
                    write!(
                        output,
                        "mode={:?} radix={:?} width={} signed={:?}",
                        s.numeric_mode,
                        s.radix,
                        s.word_config.size.bits(),
                        s.word_config.signed_mode
                    )?;
                    for i in 0..6 {
                        write!(
                            output,
                            " F{i}={}",
                            u8::from(s.flags.get(i).map_err(LineError::Calculation)?)
                        )?;
                    }
                    writeln!(
                        output,
                        " phase={:?} lift={:?} registers={}",
                        snapshot.entry_phase,
                        s.stack_lift,
                        s.registers.len()
                    )?;
                }
                Action::Help => writeln!(
                    output,
                    "Numbers push values. Use + sub * / for arithmetic, keys \"5 enter 3 +\" for key entry, stack/status to inspect, exit to quit."
                )?,
            }
        }
        Ok(RunControl::Continue)
    }
}
impl From<SessionError> for LineError {
    fn from(e: SessionError) -> Self {
        match e {
            SessionError::Input(i) => Self::Input(i),
            SessionError::Calculation(c) => Self::Calculation(c),
        }
    }
}
impl From<LiteralEntryError> for LineError {
    fn from(e: LiteralEntryError) -> Self {
        match e {
            LiteralEntryError::Literal(l) => Self::Literal(l),
            LiteralEntryError::Session(s) => s.into(),
        }
    }
}
impl From<io::Error> for LineError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}
mod parser;
#[cfg(test)]
mod tests;
