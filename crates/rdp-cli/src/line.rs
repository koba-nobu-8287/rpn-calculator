use crate::{enter_literal,LiteralEntryError,LiteralError};
use rdp_core::*;
use rdp_input::*;
use std::io::{self,Write};
#[derive(Debug)] pub enum LineError{Syntax(CliError),Literal(LiteralError),Input(InputError),Calculation(CalcError),Io(io::Error)}
#[derive(Debug)] pub enum CliError{UnknownCommand(String),MissingArgument(String),InvalidArgument(String),LineTooLong}
#[derive(Debug,PartialEq,Eq)] pub enum RunControl{Continue,Exit}
pub struct CliSession{hp:HpInputSession,pending:Option<String>}
impl CliSession{
pub fn new(calculator:Calculator)->Self{Self{hp:HpInputSession::new(calculator),pending:None}}
pub fn snapshot(&self)->InputSnapshot{self.hp.snapshot()}
pub fn run_line(&mut self,_:&str,_:&mut impl Write,_:bool)->Result<RunControl,LineError>{todo!()}
}
#[cfg(test)]mod tests;
