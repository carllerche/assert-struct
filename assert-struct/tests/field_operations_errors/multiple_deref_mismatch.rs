#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct TestStruct {
    double_box: Box<Box<i32>>,
    single_box: Box<String>,
}

pub fn test_case() {
    let test = TestStruct {
        double_box: Box::new(Box::new(42)),
        single_box: Box::new("test".to_string()),
    };

    assert_struct!(test, TestStruct {
        **double_box: 99,      // Should be 42, will fail
        *single_box: "test",
    });
}