use crate::{LiteralEntryError, LiteralError, enter_literal};
use rpn_core::*;
use rpn_input::*;
use std::io::{self, Write};
pub(crate) const MAX_LINE_BYTES: usize = 64 * 1024;
#[derive(Debug)]
pub enum LineError {
    Syntax(CliError),
    Literal(LiteralError),
    Input(InputError),
    Calculation(CalcError),
    Io(io::Error),
}
#[derive(Debug, Clone, PartialEq, Eq)]
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
        self.execute_line(line, output, interactive, false)
    }
    pub(crate) fn repl_line(
        &mut self,
        line: &str,
        output: &mut impl Write,
    ) -> Result<RunControl, LineError> {
        self.execute_line(line, output, true, true)
    }
    fn print_x(snap: &InputSnapshot, output: &mut impl Write) -> Result<(), LineError> {
        let s = &snap.calculator;
        let x = format_number(
            s.stack[0],
            s.radix,
            s.numeric_mode,
            &s.word_config,
            &s.flags,
        )
        .map_err(LineError::Calculation)?;
        if let Some(buf) = &snap.input_buffer {
            writeln!(output, "{x} [{buf}]")?;
        } else {
            writeln!(output, "{x}")?;
        }
        Ok(())
    }
    pub(crate) fn finish_batch(&mut self, output: &mut impl Write) -> Result<(), LineError> {
        let snap = self.snapshot();
        if snap.entry_phase == EntryPhase::Editing {
            self.hp.apply(InputEvent::Command(Command::SetRadix(
                snap.calculator.radix,
            )))?;
        }
        Self::print_x(&self.snapshot(), output)
    }
    fn execute_line(
        &mut self,
        line: &str,
        output: &mut impl Write,
        interactive: bool,
        echo_x: bool,
    ) -> Result<RunControl, LineError> {
        use parser::Action;
        let actions = parser::parse_line(line).map_err(LineError::Syntax)?;
        if !interactive && actions.iter().any(Action::interactive_only) {
            return Err(LineError::Syntax(CliError::InvalidArgument(
                "interactive command in batch".into(),
            )));
        }
        let show_result = echo_x
            && actions
                .last()
                .is_some_and(|action| !action.interactive_only());
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
                    let snap = self.snapshot();
                    let s = &snap.calculator;
                    for (label, value) in ["T", "Z", "Y"]
                        .into_iter()
                        .zip(s.stack[1..].iter().rev().copied())
                    {
                        let formatted =
                            format_number(value, s.radix, s.numeric_mode, &s.word_config, &s.flags)
                                .map_err(LineError::Calculation)?;
                        writeln!(output, "{label}: {formatted}")?;
                    }
                    write!(output, "X: ")?;
                    Self::print_x(&snap, output)?;
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
                    "Numbers push values. Use + sub * / for arithmetic, keys \"5 enter 3 +\" for key entry, oneszero canonical|raw for one's-complement arithmetic, stack/status to inspect, exit to quit."
                )?,
            }
        }
        if show_result {
            Self::print_x(&self.snapshot(), output)?;
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

impl LineError {
    pub fn exit_code(&self) -> u8 {
        match self {
            Self::Calculation(_) | Self::Input(_) => 1,
            Self::Literal(_) | Self::Syntax(_) => 2,
            Self::Io(_) => 3,
        }
    }
}
impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::InvalidArgument(s) => write!(f, "invalid argument: {s}"),
            CliError::MissingArgument(s) => write!(f, "missing argument for {s}"),
            CliError::UnknownCommand(s) => write!(f, "unknown command: {s}"),
            CliError::LineTooLong => f.write_str("line exceeds 64 KiB"),
        }
    }
}
impl std::error::Error for CliError {}
impl std::fmt::Display for LineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syntax(e) => e.fmt(f),
            Self::Calculation(e) => write!(f, "calculation: {e}"),
            Self::Input(e) => write!(f, "key input: {e}"),
            Self::Literal(e) => write!(f, "numeric literal: {e}"),
            Self::Io(e) => write!(f, "I/O: {e}"),
        }
    }
}
impl std::error::Error for LineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::Syntax(e) => e,
            Self::Literal(e) => e,
            Self::Input(e) => e,
            Self::Calculation(e) => e,
            Self::Io(e) => e,
        })
    }
}
