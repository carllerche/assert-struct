// Test field operations like dereferencing, method calls, and nested access

use assert_struct::assert_struct;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug)]
#[allow(dead_code)]
struct TestStruct {
    boxed_value: Box<i32>,
    rc_value: Rc<String>,
    arc_value: Arc<u32>,
    tuple_with_box: (String, Box<i32>),
    normal_value: i32,
}

#[test]
fn test_deref_simple() {
    let test = TestStruct {
        boxed_value: Box::new(42),
        rc_value: Rc::new("hello".to_string()),
        arc_value: Arc::new(100),
        tuple_with_box: ("test".to_string(), Box::new(99)),
        normal_value: 200,
    };

    // Test basic dereferencing
    assert_struct!(test, TestStruct {
        normal_value: 200,
        *boxed_value: 42,
        ..
    });
}

#[test]
fn test_tuple_indexed_deref() {
    let test = TestStruct {
        boxed_value: Box::new(42),
        rc_value: Rc::new("hello".to_string()),
        arc_value: Arc::new(100),
        tuple_with_box: ("test".to_string(), Box::new(99)),
        normal_value: 200,
    };

    // Test tuple with indexed dereferencing
    assert_struct!(test, TestStruct {
        tuple_with_box: ("test", *1: 99),
        ..
    });
}

#[test]
fn test_mixed_tuple_syntax() {
    let test = TestStruct {
        boxed_value: Box::new(42),
        rc_value: Rc::new("hello".to_string()),
        arc_value: Arc::new(100),
        tuple_with_box: ("test".to_string(), Box::new(99)),
        normal_value: 200,
    };

    // Test mixed positional and indexed syntax
    assert_struct!(test, TestStruct {
        tuple_with_box: ("test", *1: 99),  // Mixed: positional + indexed with deref
        ..
    });
}
