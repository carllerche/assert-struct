#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct TestStruct {
    boxed_option: Box<Option<String>>,
}

pub fn test_case() {
    let test = TestStruct {
        boxed_option: Box::new(Some("hello".to_string())),
    };

    assert_struct!(test, TestStruct {
        *boxed_option: Some("goodbye"),  // Should be "hello", will fail
    });
}