mod literal;
pub use literal::LiteralError;
use rpn_input::{HpInputSession, SessionError};
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LiteralEntryError {
    Literal(LiteralError),
    Session(SessionError),
}
impl std::fmt::Display for LiteralEntryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Literal(e) => write!(f, "numeric literal: {e}"),
            Self::Session(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for LiteralEntryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(match self {
            Self::Literal(e) => e,
            Self::Session(e) => e,
        })
    }
}

/// Parses one complete numeric token under the current calculator settings and pushes its value.
pub fn enter_literal(session: &mut HpInputSession, text: &str) -> Result<(), LiteralEntryError> {
    let state = session.snapshot().calculator;
    let context = literal::ParseContext {
        radix: state.radix,
        mode: state.numeric_mode,
        word_config: state.word_config,
    };
    let value = literal::parse_literal(text, context).map_err(LiteralEntryError::Literal)?;
    session
        .apply(rpn_input::InputEvent::Command(rpn_core::Command::Push(
            value,
        )))
        .map_err(LiteralEntryError::Session)
}
pub mod line;
pub mod terminal;
