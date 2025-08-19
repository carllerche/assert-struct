use assert_struct::assert_struct;

#[derive(Debug)]
struct Data {
    values: Vec<i32>,
    names: Vec<String>,
    matrix: Vec<Vec<i32>>,
}

#[derive(Debug)]
struct NestedData {
    items: Vec<Item>,
}

#[derive(Debug)]
struct Item {
    tags: Vec<String>,
    scores: Vec<f64>,
}

#[test]
fn test_basic_index_operations() {
    let data = Data {
        values: vec![10, 20, 30],
        names: vec!["alice".to_string(), "bob".to_string(), "charlie".to_string()],
        matrix: vec![vec![1, 2], vec![3, 4], vec![5, 6]],
    };

    // Basic index access
    assert_struct!(data, Data {
        values[0]: 10,
        values[1]: 20,
        names[0]: "alice",
        names[2]: "charlie",
        ..
    });
}

#[test]
fn test_index_with_comparisons() {
    let data = Data {
        values: vec![5, 15, 25, 35],
        names: vec!["test".to_string()],
        matrix: vec![],
    };

    assert_struct!(data, Data {
        values[0]: < 10,
        values[1]: >= 10,
        values[2]: > 20,
        values[3]: == 35,
        ..
    });
}

#[test]
fn test_nested_index_operations() {
    let data = Data {
        values: vec![],
        names: vec![],
        matrix: vec![vec![1, 2, 3], vec![4, 5, 6]],
    };

    assert_struct!(data, Data {
        matrix[0][1]: 2,
        matrix[1][0]: 4,
        matrix[1][2]: 6,
        ..
    });
}

#[test]
fn test_chained_field_and_index() {
    let data = NestedData {
        items: vec![
            Item {
                tags: vec!["rust".to_string(), "test".to_string()],
                scores: vec![9.5, 8.7, 9.2],
            },
            Item {
                tags: vec!["code".to_string()],
                scores: vec![8.0],
            },
        ],
    };

    assert_struct!(data, NestedData {
        items[0].tags[0]: "rust",
        items[0].tags[1]: "test",
        items[0].scores[0]: > 9.0,
        items[1].tags[0]: "code",
        items[1].scores[0]: == 8.0,
        ..
    });
}

#[test]
fn test_index_with_method_calls() {
    let data = Data {
        values: vec![100, 200],
        names: vec!["hello".to_string(), "world".to_string()],
        matrix: vec![],
    };

    assert_struct!(data, Data {
        names[0].len(): 5,
        names[1].starts_with("wor"): true,
        ..
    });
}

#[test]
fn test_mixed_operations() {
    let data = Data {
        values: vec![1, 2, 3, 4, 5],
        names: vec!["test".to_string()],
        matrix: vec![],
    };

    // Mix of regular field access, index operations, and method calls
    assert_struct!(data, Data {
        values.len(): 5,
        values[0]: 1,
        values[4]: 5,
        names.len(): 1,
        names[0]: "test",
        names[0].len(): 4,
        ..
    });
}

#[test]
fn test_index_with_variables() {
    let data = Data {
        values: vec![10, 20, 30],
        names: vec!["test".to_string()],
        matrix: vec![],
    };

    let index = 1;
    let expected = 20;

    assert_struct!(data, Data {
        values[index]: expected,
        ..
    });
}

#[test]
fn test_index_with_expressions() {
    let data = Data {
        values: vec![1, 2, 3, 4, 5],
        names: vec!["test".to_string()],
        matrix: vec![],
    };

    assert_struct!(data, Data {
        values[2 + 1]: 4,
        values[data.values.len() - 1]: 5,
        ..
    });
}

// Error test cases using snapshot testing

// Error message tests using the error_message_test! macro
mod util;

error_message_test!("index_operations_errors/index_value_mismatch.rs", index_value_mismatch);
error_message_test!("index_operations_errors/nested_index_mismatch.rs", nested_index_mismatch);
error_message_test!("index_operations_errors/index_comparison_failure.rs", index_comparison_failure);
error_message_test!("index_operations_errors/chained_index_field_mismatch.rs", chained_index_field_mismatch);