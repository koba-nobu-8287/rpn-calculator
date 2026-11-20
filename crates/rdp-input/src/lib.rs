use rdp_core::*;
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum EntryPhase {Idle,Editing}
#[derive(Debug,Clone,Copy,PartialEq,Eq)]
pub enum EntryPart {Mantissa,Exponent}
#[derive(Debug,Clone,Copy)]
pub enum Key {Digit(char),DecimalPoint,Exponent,Backspace,ChangeSign}
pub enum InputEvent {Key(Key), Command(Command)}
#[derive(Debug,Clone,PartialEq)]
pub enum InputError {InvalidDigit{digit:char,radix:Radix},InvalidNumber,IncompleteEntry,InputInProgress,InputTooLong,InvariantViolation}
#[derive(Debug,Clone,PartialEq)]
pub enum SessionError {Input(InputError),Calculation(CalcError)}
#[derive(Debug,Clone,PartialEq)]
pub struct InputSnapshot {pub calculator:CalculatorSnapshot,pub entry_phase:EntryPhase,pub input_buffer:Option<String>,pub entry_part:EntryPart}
#[derive(Clone)]
pub struct HpInputSession { calculator:Calculator, buffer:String }
impl HpInputSession {
pub fn new(calculator:Calculator)->Self{Self{calculator,buffer:String::new()}}
pub fn snapshot(&self)->InputSnapshot{todo!()}
pub fn apply(&mut self,_event:InputEvent)->Result<(),SessionError>{todo!()}
}
