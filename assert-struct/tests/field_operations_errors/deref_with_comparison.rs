#![allow(dead_code)]
use assert_struct::assert_struct;
use std::sync::Arc;

#[derive(Debug)]
struct TestStruct {
    arc_value: Arc<i32>,
    normal: u32,
}

pub fn test_case() {
    let test = TestStruct {
        arc_value: Arc::new(50),
        normal: 100,
    };

    assert_struct!(test, TestStruct {
        *arc_value: > 60,  // Should be > 60 but actual is 50, will fail
        normal: 100,
    });
}