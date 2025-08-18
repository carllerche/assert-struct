#![allow(dead_code)]
use assert_struct::assert_struct;
use std::rc::Rc;

#[derive(Debug)]
struct NestedStruct {
    inner: InnerStruct,
}

#[derive(Debug)]
struct InnerStruct {
    rc_value: Rc<String>,
    normal: u32,
}

pub fn test_case() {
    let test = NestedStruct {
        inner: InnerStruct {
            rc_value: Rc::new("hello".to_string()),
            normal: 42,
        },
    };

    assert_struct!(test, NestedStruct {
        inner: InnerStruct {
            *rc_value: "goodbye",  // Should be "hello", will fail
            normal: 42,
        },
    });
}