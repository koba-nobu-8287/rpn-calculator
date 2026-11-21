use rpn_core::*;
use rpn_input::*;
#[test]
fn edit_hex_and_backspace() {
    let mut s = HpInputSession::new(Calculator::new());
    for (key, buf) in [
        (Key::Digit('a'), "A"),
        (Key::Digit('0'), "A0"),
        (Key::Backspace, "A"),
        (Key::Backspace, "0"),
    ] {
        s.apply(InputEvent::Key(key)).unwrap();
        assert_eq!(s.snapshot().input_buffer.as_deref(), Some(buf));
    }
}
