#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct TestStruct {
    boxed_vec: Box<Vec<i32>>,
}

pub fn test_case() {
    let test = TestStruct {
        boxed_vec: Box::new(vec![1, 2, 3]),
    };

    assert_struct!(test, TestStruct {
        *boxed_vec: [1, 2, 4],  // Should be [1, 2, 3], will fail
    });
}