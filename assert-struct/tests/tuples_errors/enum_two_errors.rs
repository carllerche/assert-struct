#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
enum Message {
    Pair(u32, String),
}

#[derive(Debug)]
struct MessageQueue {
    id: u32,
    current: Message,
}

pub fn test_case() {
    let queue = MessageQueue {
        id: 1,
        current: Message::Pair(50, "actual".to_string()),
    };

    assert_struct!(
        queue,
        MessageQueue {
            id: 1,
            current: Message::Pair(100, "expected"),  // Both fields wrong
        }
    );
}