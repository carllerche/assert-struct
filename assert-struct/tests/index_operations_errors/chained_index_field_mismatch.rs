#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Item {
    tags: Vec<String>,
}

#[derive(Debug)]
struct Data {
    items: Vec<Item>,
}

pub fn test_case() {
    let data = Data {
        items: vec![
            Item {
                tags: vec!["rust".to_string(), "test".to_string()],
            },
        ],
    };

    assert_struct!(data, Data {
        items[0].tags[0]: "python",  // Should be "rust", will fail
        ..
    });
}