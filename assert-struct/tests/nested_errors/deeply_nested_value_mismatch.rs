#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Level3 {
    name: String,
    value: i32,
}

#[derive(Debug)]
struct Level2 {
    id: u32,
    nested: Level3,
}

#[derive(Debug)]
struct Level1 {
    tag: String,
    inner: Level2,
}

pub fn test_case() {
    let data = Level1 {
        tag: "test".to_string(),
        inner: Level2 {
            id: 42,
            nested: Level3 {
                name: "actual".to_string(),
                value: 100,
            },
        },
    };

    assert_struct!(data, Level1 {
        tag: "test",
        inner: Level2 {
            id: 42,
            nested: Level3 {
                name: "expected",  // Line 38 - should report this line
                value: 100,
            },
        },
    });
}