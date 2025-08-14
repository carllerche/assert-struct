use assert_struct::assert_struct;

#[derive(Debug, PartialEq)]
enum Message {
    Text(String),
    Data(String, Vec<u8>),
    Complex(u32, String, bool),
}

#[derive(Debug)]
struct MessageQueue {
    current: Message,
    priority: u8,
}

pub fn test_case() {
    let msg = MessageQueue {
        current: Message::Complex(42, "wrong".to_string(), true),
        priority: 3,
    };

    assert_struct!(
        msg,
        MessageQueue {
            current: Message::Complex(42, "test", true),
            priority: 3,
        }
    );
}