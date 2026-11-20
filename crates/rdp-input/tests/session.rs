use rdp_core::*;
use rdp_input::*;
#[test]
fn failed_division_resumes_same_entry(){
let mut s=HpInputSession::new(Calculator::new());
s.apply(InputEvent::Key(Key::Digit('5'))).unwrap();
s.apply(InputEvent::Command(Command::Enter)).unwrap();
s.apply(InputEvent::Key(Key::Digit('0'))).unwrap();
let before=s.snapshot();
assert_eq!(s.apply(InputEvent::Command(Command::Divide)),Err(SessionError::Calculation(CalcError::DivisionByZero)));
assert_eq!(s.snapshot(),before);
s.apply(InputEvent::Key(Key::Digit('2'))).unwrap();
assert_eq!(s.snapshot().input_buffer.as_deref(),Some("02"));
assert_eq!(s.snapshot().calculator.stack[1].as_integer().unwrap().bits(),5);
}
