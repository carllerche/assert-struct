#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct TestStruct {
    boxed_value: Box<i32>,
    normal_value: i32,
}

pub fn test_case() {
    let test = TestStruct {
        boxed_value: Box::new(42),
        normal_value: 200,
    };

    assert_struct!(test, TestStruct {
        *boxed_value: 99,  // Should be 42, will fail
        normal_value: 200,
        ..
    });
}