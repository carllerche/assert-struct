#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct TestStruct {
    tuple_with_box: (String, Box<i32>),
    normal_value: i32,
}

pub fn test_case() {
    let test = TestStruct {
        tuple_with_box: ("test".to_string(), Box::new(99)),
        normal_value: 200,
    };

    assert_struct!(test, TestStruct {
        tuple_with_box: ("test", *1: 42),  // Should be 99, will fail
        normal_value: 200,
        ..
    });
}