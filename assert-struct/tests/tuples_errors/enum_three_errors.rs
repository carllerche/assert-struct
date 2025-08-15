use assert_struct::assert_struct;

#[derive(Debug)]
enum Message {
    Triple(i32, String, bool),
}

#[derive(Debug)]
struct MessageQueue {
    id: u32,
    current: Message,
}

pub fn test_case() {
    let queue = MessageQueue {
        id: 2,
        current: Message::Triple(42, "actual".to_string(), true),
    };

    assert_struct!(
        queue,
        MessageQueue {
            id: 2,
            current: Message::Triple(100, "expected", false),  // All three fields wrong
        }
    );
}