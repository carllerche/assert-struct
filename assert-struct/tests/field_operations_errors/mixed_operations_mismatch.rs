#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct TestStruct {
    boxed_value: Box<i32>,
    tuple_mix: (String, Box<u32>, i32),
}

pub fn test_case() {
    let test = TestStruct {
        boxed_value: Box::new(42),
        tuple_mix: ("hello".to_string(), Box::new(100), 25),
    };

    assert_struct!(test, TestStruct {
        *boxed_value: 42,
        tuple_mix: ("hello", *1: 200, 30),  // Both 1: and 2: will fail
    });
}