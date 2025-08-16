#![allow(dead_code, unused_variables)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct TestStruct {
    name: String,
    value: i32,
    data: Option<String>,
    items: Vec<i32>,
    tuple: (i32, String, bool),
}

#[test]
fn test_wildcard_in_struct_field() {
    let test = TestStruct {
        name: "test".to_string(),
        value: 42,
        data: Some("data".to_string()),
        items: vec![1, 2, 3],
        tuple: (10, "hello".to_string(), true),
    };

    // Test wildcard for struct field - we don't care about the value
    assert_struct!(
        test,
        TestStruct {
            name: _,
            value: 42,
            data: Some("data"),
            items: [1, 2, 3],
            tuple: (10, "hello", true),
        }
    );
}

#[test]
fn test_wildcard_in_option() {
    let test = TestStruct {
        name: "test".to_string(),
        value: 42,
        data: Some("any value".to_string()),
        items: vec![],
        tuple: (0, "".to_string(), false),
    };

    // Test Some(_) to check is_some without caring about value
    assert_struct!(
        test,
        TestStruct {
            name: "test",
            value: 42,
            data: Some(_),
            items: [],
            tuple: (0, "", false),
        }
    );
}

#[test]
fn test_wildcard_in_tuple() {
    let test = TestStruct {
        name: "test".to_string(),
        value: 42,
        data: None,
        items: vec![],
        tuple: (999, "ignored".to_string(), true),
    };

    // Test wildcards in tuple - only care about the boolean
    assert_struct!(
        test,
        TestStruct {
            name: "test",
            value: 42,
            data: None,
            items: [],
            tuple: (_, _, true),
        }
    );
}

#[test]
fn test_wildcard_in_slice() {
    let test = TestStruct {
        name: "test".to_string(),
        value: 42,
        data: None,
        items: vec![10, 20, 30],
        tuple: (0, "".to_string(), false),
    };

    // Test wildcard in slice pattern
    assert_struct!(
        test,
        TestStruct {
            name: "test",
            value: 42,
            data: None,
            items: [10, _, 30],
            tuple: (0, "", false),
        }
    );
}

#[test]
fn test_multiple_wildcards() {
    let test = TestStruct {
        name: "anything".to_string(),
        value: 999,
        data: Some("whatever".to_string()),
        items: vec![1, 2, 3, 4, 5],
        tuple: (123, "xyz".to_string(), false),
    };

    // Test multiple wildcards - only care about specific fields
    assert_struct!(
        test,
        TestStruct {
            name: _,
            value: _,
            data: Some(_),
            items: [1, _, 3, _, 5],
            tuple: (_, _, false),
        }
    );
}

#[test]
#[should_panic(expected = "mismatch")]
fn test_wildcard_with_none_fails() {
    let test = TestStruct {
        name: "test".to_string(),
        value: 42,
        data: None,
        items: vec![],
        tuple: (0, "".to_string(), false),
    };

    // This should fail - Some(_) expects Some, but got None
    assert_struct!(
        test,
        TestStruct {
            name: "test",
            value: 42,
            data: Some(_),
            items: [],
            tuple: (0, "", false),
        }
    );
}
