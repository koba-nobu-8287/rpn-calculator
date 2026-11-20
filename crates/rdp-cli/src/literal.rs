use rdp_core::{Number, NumericMode, Radix, WordConfig};
#[derive(Clone,Copy)]
pub(crate) struct ParseContext {pub(crate) radix:Radix,pub(crate) mode:NumericMode,pub(crate) word_config:WordConfig}
#[derive(Debug,PartialEq,Eq)]
pub enum LiteralError{InputTooLong,InvalidDigit{digit:char,radix:Radix},InvalidNumber,FloatingPointOutOfRange}
fn parse_literal(_: &str,_:ParseContext)->Result<Number,LiteralError>{todo!()}
#[cfg(test)] mod tests;
