// Test field operations like dereferencing, method calls, and nested access

use assert_struct::assert_struct;
use std::rc::Rc;
use std::sync::Arc;

#[macro_use]
mod util;

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

#[test]
fn test_root_field_access_lhs() {
    let test = TestStruct {
        boxed_value: Box::new(42),
        rc_value: Rc::new("hello".to_string()),
        arc_value: Arc::new(100),
        tuple_with_box: ("test".to_string(), Box::new(99)),
        normal_value: 200,
    };

    // Root field access
    assert_struct!(test.normal_value, 200);
    assert_struct!(*test.boxed_value, 42);
    assert_struct!(*test.rc_value, "hello");
}

#[derive(Debug)]
#[allow(dead_code)]
enum FieldEvent {
    Data(TestStruct),
    Many(Vec<TestStruct>),
}

#[test]
fn test_nested_variant_field_ops_lhs() {
    let event = FieldEvent::Data(TestStruct {
        boxed_value: Box::new(10),
        rc_value: Rc::new("inner".to_string()),
        arc_value: Arc::new(20),
        tuple_with_box: ("t".to_string(), Box::new(30)),
        normal_value: 40,
    });

    assert_struct!(event, FieldEvent::Data(TestStruct {
        *boxed_value: 10,
        *rc_value: "inner",
        ..
    }));
}
// Error message tests using snapshot testing
error_message_test!(
    "field_operations_errors/deref_field_mismatch.rs",
    deref_field_mismatch
);
error_message_test!(
    "field_operations_errors/tuple_indexed_deref_mismatch.rs",
    tuple_indexed_deref_mismatch
);
error_message_test!(
    "field_operations_errors/nested_deref_mismatch.rs",
    nested_deref_mismatch
);
error_message_test!(
    "field_operations_errors/multiple_deref_mismatch.rs",
    multiple_deref_mismatch
);
error_message_test!(
    "field_operations_errors/mixed_operations_mismatch.rs",
    mixed_operations_mismatch
);
error_message_test!(
    "field_operations_errors/deref_with_comparison.rs",
    deref_with_comparison
);
error_message_test!(
    "field_operations_errors/enum_with_deref_mismatch.rs",
    enum_with_deref_mismatch
);
error_message_test!(
    "field_operations_errors/slice_with_deref_mismatch.rs",
    slice_with_deref_mismatch
);
