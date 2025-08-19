#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Data {
    values: Vec<i32>,
}

pub fn test_case() {
    let data = Data {
        values: vec![5, 15, 25],
    };

    assert_struct!(data, Data {
        values[1]: > 20,  // Should be > 20, but actual is 15
        ..
    });
}