#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
enum Message {
    Quad(u8, u8, u8, u8),
}

#[derive(Debug)]
struct MessageQueue {
    id: u32,
    current: Message,
}

pub fn test_case() {
    let queue = MessageQueue {
        id: 3,
        current: Message::Quad(10, 20, 30, 40),
    };

    assert_struct!(
        queue,
        MessageQueue {
            id: 3,
            current: Message::Quad(1, 2, 3, 4),  // All four fields wrong
        }
    );
}