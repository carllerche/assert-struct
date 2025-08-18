// Test method call patterns for assert_struct!

use assert_struct::assert_struct;
use std::collections::HashMap;

#[derive(Debug)]
#[allow(dead_code)]
struct TestData {
    text: String,
    numbers: Vec<i32>,
    maybe_value: Option<String>,
    result_value: Result<i32, String>,
    map: HashMap<String, i32>,
    tuple_data: (Vec<String>, String, Option<i32>),
}

#[test]
fn test_basic_method_calls() {
    let data = TestData {
        text: "hello world".to_string(),
        numbers: vec![1, 2, 3],
        maybe_value: Some("test".to_string()),
        result_value: Ok(42),
        map: {
            let mut m = HashMap::new();
            m.insert("key1".to_string(), 10);
            m.insert("key2".to_string(), 20);
            m
        },
        tuple_data: (
            vec!["a".to_string(), "b".to_string()],
            "middle".to_string(),
            Some(99),
        ),
    };

    // Test basic method calls
    assert_struct!(data, TestData {
        text.len(): 11,
        numbers.len(): 3,
        maybe_value.is_some(): true,
        result_value.is_ok(): true,
        map.len(): 2,
        ..
    });
}

#[test]
fn test_tuple_method_calls() {
    let data = TestData {
        text: "hello".to_string(),
        numbers: vec![1, 2, 3, 4, 5],
        maybe_value: None,
        result_value: Err("error".to_string()),
        map: HashMap::new(),
        tuple_data: (
            vec!["first".to_string(), "second".to_string()],
            "middle".to_string(),
            None,
        ),
    };

    // Test method calls on tuple elements
    assert_struct!(data, TestData {
        tuple_data: (
            0.len(): 2,          // vec.len() == 2
            1.len(): 6,          // "middle".len() == 6
            2.is_none(): true,   // Option::is_none() == true
        ),
        ..
    });
}

#[test]
fn test_method_calls_with_comparisons() {
    let data = TestData {
        text: "hello world".to_string(),
        numbers: vec![1, 2, 3, 4, 5, 6, 7],
        maybe_value: Some("test value".to_string()),
        result_value: Ok(42),
        map: {
            let mut m = HashMap::new();
            m.insert("a".to_string(), 1);
            m.insert("b".to_string(), 2);
            m.insert("c".to_string(), 3);
            m
        },
        tuple_data: (vec!["x".to_string()], "y".to_string(), Some(123)),
    };

    // Test method calls with comparison operators
    assert_struct!(data, TestData {
        text.len(): > 10,
        numbers.len(): >= 5,
        map.len(): == 3,
        tuple_data: (
            0.len(): < 5,
            1.len(): == 1,
            _,
        ),
        ..
    });
}

#[test]
fn test_method_calls_with_arguments() {
    let data = TestData {
        text: "hello world".to_string(),
        numbers: vec![1, 2, 3],
        maybe_value: Some("test".to_string()),
        result_value: Ok(42),
        map: {
            let mut m = HashMap::new();
            m.insert("key1".to_string(), 10);
            m
        },
        tuple_data: (vec!["abc".to_string()], "def".to_string(), Some(99)),
    };

    // Test method calls with arguments
    assert_struct!(data, TestData {
        text.contains("world"): true,
        numbers.contains(&2): true,
        map.contains_key("key1"): true,
        ..
    });

    // Separate test for different method on the same field
    assert_struct!(data, TestData {
        text.starts_with("hello"): true,
        ..
    });
}
