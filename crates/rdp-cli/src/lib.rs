mod literal;
pub use literal::LiteralError;
use rdp_input::{HpInputSession, SessionError};
#[derive(Debug, PartialEq)]
pub enum LiteralEntryError {
    Literal(LiteralError),
    Session(SessionError),
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
        .apply(rdp_input::InputEvent::Command(rdp_core::Command::Push(
            value,
        )))
        .map_err(LiteralEntryError::Session)
}
pub mod line;
