use super::*;
fn session()->CliSession{CliSession::new(Calculator::new())}
#[test] fn syntax_checks_the_entire_line(){
 for line in ["1 unknown","1 sf 6","1 wsize 0","1 maskl 65","1 signed","1 keys \"1","1 keys \"0xff\"","1 keys \"keys\"","1 sto -1"]{
 let mut s=session();let before=s.snapshot();assert!(matches!(s.run_line(line,&mut Vec::new(),true),Err(LineError::Syntax(_))),"{line}");assert_eq!(s.snapshot(),before);}
}
#[test] fn syntax_and_runtime_failures_have_different_commit_boundaries(){
 for (line,want) in [("5 0 /",0),("5 0xG",5),("5 lj",5)]{
 let mut s=session();let error=s.run_line(line,&mut Vec::new(),true).unwrap_err();
 assert!(matches!((line,error),("5 0 /",LineError::Calculation(CalcError::DivisionByZero))|("5 0xG",LineError::Literal(LiteralError::InvalidDigit{..}))|("5 lj",LineError::Calculation(CalcError::UnsupportedOperation))));
 assert_eq!(s.snapshot().calculator.stack[0].as_integer().unwrap().bits(),want);}
}
#[test] fn reserved_words_and_key_subtraction(){
 for (line,want) in [("HEX ff DeC 10 ADD # ignored",265),("dec 10 - sub",10),("dec keys \"7 enter 2 -\"",5),("dec keys \"1 2\"",12)]{
 let mut s=session();s.run_line(line,&mut Vec::new(),true).unwrap();assert_eq!(s.snapshot().calculator.stack[0].as_integer().unwrap().bits(),want);}
}
#[test] fn batch_preflight_and_exit(){
 let mut s=session();let before=s.snapshot();assert!(matches!(s.run_line("5 0 / stack",&mut Vec::new(),false),Err(LineError::Syntax(_))));assert_eq!(s.snapshot(),before);
 assert_eq!(s.run_line("exit 5",&mut Vec::new(),true).unwrap(),RunControl::Exit);assert_eq!(s.snapshot(),before);
}
