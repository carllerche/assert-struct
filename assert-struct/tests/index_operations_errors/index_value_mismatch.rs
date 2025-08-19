#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Data {
    values: Vec<i32>,
}

pub fn test_case() {
    let data = Data {
        values: vec![10, 20, 30],
    };

    assert_struct!(data, Data {
        values[1]: 25,  // Should be 20, will fail
        ..
    });
}