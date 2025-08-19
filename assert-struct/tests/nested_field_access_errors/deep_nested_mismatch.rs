#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Level3 {
    value: String,
}

#[derive(Debug)]
struct Level2 {
    level3: Level3,
}

#[derive(Debug)]
struct Level1 {
    level2: Level2,
}

pub fn test_case() {
    let data = Level1 {
        level2: Level2 {
            level3: Level3 {
                value: "actual".to_string(),
            },
        },
    };

    assert_struct!(data, Level1 {
        level2.level3.value: "expected",  // Should be "actual", will fail
        ..
    });
}