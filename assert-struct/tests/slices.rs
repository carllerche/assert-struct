use assert_struct::assert_struct;

#[macro_use]
mod util;

#[derive(Debug)]
struct Container {
    items: Vec<u32>,
    names: Vec<String>,
    data: Vec<i32>,
}

#[test]
fn test_exact_slice_match() {
    let container = Container {
        items: vec![1, 2, 3],
        names: vec!["a".to_string(), "b".to_string()],
        data: vec![10, 20, 30],
    };

    assert_struct!(
        container,
        Container {
            items: [1, 2, 3],
            names: ["a", "b"],
            data: [10, 20, 30],
        }
    );
}

#[test]
fn test_slice_with_comparisons() {
    let container = Container {
        items: vec![5, 15, 25],
        names: vec!["test".to_string()],
        data: vec![-10, 0, 10],
    };

    // Comparison operators in slices
    assert_struct!(container, Container {
        items: [> 0, < 20, == 25],  // Each element with its own matcher
        names: ["test"],
        data: [< 0, == 0, > 0],
    });
}

#[test]
fn test_slice_with_partial_matching() {
    let container = Container {
        items: vec![1, 2, 3, 4, 5],
        names: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        data: vec![100],
    };

    // Partial slice matching with ..
    assert_struct!(
        container,
        Container {
            items: [1, 2, ..], // First two elements match, rest ignored
            names: ["a", "b", ..],
            data: [100],
        }
    );
}

#[test]
fn test_slice_partial_with_comparisons() {
    let container = Container {
        items: vec![5, 15, 25, 35, 45],
        names: vec![
            "first".to_string(),
            "second".to_string(),
            "third".to_string(),
        ],
        data: vec![10, 20, 30, 40, 50],
    };

    // Combine partial matching with comparisons
    assert_struct!(container, Container {
        items: [> 0, < 20, ..],  // Check first two with comparisons, ignore rest
        names: ["first", ..],    // Check first, ignore rest
        data: [10, .., 50],      // Check first and last
    });
}

#[test]
fn test_slice_head_tail_patterns() {
    let container = Container {
        items: vec![1, 2, 3, 4, 5],
        names: vec![
            "first".to_string(),
            "middle".to_string(),
            "last".to_string(),
        ],
        data: vec![10, 20, 30, 40],
    };

    // Head and tail matching
    assert_struct!(
        container,
        Container {
            items: [1, .., 5], // First and last
            names: ["first", .., "last"],
            data: [10, .., 40],
        }
    );
}

#[test]
fn test_slice_suffix_pattern() {
    let container = Container {
        items: vec![1, 2, 3, 4, 5],
        names: vec!["a".to_string(), "b".to_string(), "c".to_string()],
        data: vec![100, 200, 300],
    };

    // Check only last elements
    assert_struct!(
        container,
        Container {
            items: [.., 4, 5], // Last two elements
            names: [.., "c"],  // Last element
            data: [.., 300],   // Last element
        }
    );
}

#[test]
#[cfg(feature = "regex")]
fn test_slice_with_regex() {
    let container = Container {
        items: vec![100, 200, 300],
        names: vec![
            "alice_1".to_string(),
            "bob_2".to_string(),
            "charlie_3".to_string(),
        ],
        data: vec![1],
    };

    // Regex patterns in slices
    assert_struct!(container, Container {
        items: [>= 100, <= 200, == 300],
        names: [=~ r"^alice", =~ r"^bob", =~ r"^charlie"],
        data: [1],
    });
}

#[test]
fn test_empty_slice() {
    let container = Container {
        items: vec![],
        names: vec![],
        data: vec![],
    };

    // Empty slice patterns
    assert_struct!(
        container,
        Container {
            items: [],
            names: [],
            data: [],
        }
    );
}

#[test]
#[should_panic(expected = "mismatch")]
fn test_slice_length_mismatch() {
    let container = Container {
        items: vec![1, 2, 3],
        names: vec!["a".to_string()],
        data: vec![10],
    };

    assert_struct!(
        container,
        Container {
            items: [1, 2], // Wrong length
            names: ["a"],
            data: [10],
        }
    );
}

error_message_test!(
    "slices_errors/slice_element_mismatch.rs",
    slice_element_mismatch
);

// Advanced: nested structures in slices
#[derive(Debug, PartialEq)]
struct Item {
    id: u32,
    value: String,
}

#[derive(Debug)]
struct Inventory {
    items: Vec<Item>,
}

#[test]
fn test_slice_with_nested_structs() {
    let inventory = Inventory {
        items: vec![
            Item {
                id: 1,
                value: "sword".to_string(),
            },
            Item {
                id: 2,
                value: "shield".to_string(),
            },
        ],
    };

    assert_struct!(
        inventory,
        Inventory {
            items: vec![
                Item {
                    id: 1,
                    value: "sword".to_string()
                },
                Item {
                    id: 2,
                    value: "shield".to_string()
                },
            ],
        }
    );
}

#[test]
fn test_slice_with_nested_struct_patterns() {
    let inventory = Inventory {
        items: vec![
            Item {
                id: 1,
                value: "sword".to_string(),
            },
            Item {
                id: 2,
                value: "shield".to_string(),
            },
            Item {
                id: 3,
                value: "potion".to_string(),
            },
        ],
    };

    // Test partial matching on nested structs in slices
    assert_struct!(inventory, Inventory {
        items: [
            Item { id: 1, .. },  // Only check id
            Item { value: "shield", .. },  // Only check value
            Item { id: > 2, value: "potion" },  // Check both with comparison
        ],
    });
}

#[test]
fn test_slice_with_nested_struct_partial_slice() {
    let inventory = Inventory {
        items: vec![
            Item {
                id: 1,
                value: "sword".to_string(),
            },
            Item {
                id: 2,
                value: "shield".to_string(),
            },
            Item {
                id: 3,
                value: "potion".to_string(),
            },
            Item {
                id: 4,
                value: "bow".to_string(),
            },
        ],
    };

    // Test combining partial slice matching with partial struct matching
    assert_struct!(
        inventory,
        Inventory {
            items: [
                Item { id: 1, .. }, // First item, only check id
                Item {
                    value: "shield",
                    ..
                }, // Second item, only check value
                ..,                 // Ignore the rest
            ],
        }
    );

    // Test suffix pattern with struct matching
    assert_struct!(
        inventory,
        Inventory {
            items: [
                .., // Ignore initial items
                Item {
                    id: 4,
                    value: "bow"
                }, // Last item, check all fields
            ],
        }
    );
}

// Test with Option<Vec<T>>
#[derive(Debug)]
struct Config {
    values: Option<Vec<u32>>,
}

#[test]
fn test_option_vec_some() {
    let config = Config {
        values: Some(vec![1, 2, 3]),
    };

    assert_struct!(
        config,
        Config {
            values: Some([1, 2, 3]),
        }
    );
}

#[test]
fn test_option_vec_some_partial() {
    let config = Config {
        values: Some(vec![1, 2, 3, 4, 5]),
    };

    assert_struct!(
        config,
        Config {
            values: Some([1, 2, ..]),
        }
    );

    assert_struct!(
        config,
        Config {
            values: Some([.., 5]),
        }
    );

    assert_struct!(
        config,
        Config {
            values: Some([1, .., 5]),
        }
    );
}

#[test]
fn test_option_vec_some_with_comparisons() {
    let config = Config {
        values: Some(vec![10, 20, 30]),
    };

    assert_struct!(config, Config {
        values: Some([> 5, < 25, == 30]),
    });
}

#[test]
fn test_option_vec_some_workaround() {
    let config = Config {
        values: Some(vec![1, 2, 3]),
    };

    assert_struct!(
        config,
        Config {
            values: Some(vec![1, 2, 3]),
        }
    );
}

#[test]
fn test_option_vec_none() {
    let config = Config { values: None };

    assert_struct!(config, Config { values: None });
}

// Test with Result<Vec<T>, String>
#[derive(Debug)]
struct Response {
    data: Result<Vec<u32>, String>,
}

#[test]
fn test_result_vec_ok() {
    let response = Response {
        data: Ok(vec![1, 2, 3]),
    };

    assert_struct!(
        response,
        Response {
            data: Ok([1, 2, 3]),
        }
    );
}

#[test]
fn test_result_vec_ok_partial() {
    let response = Response {
        data: Ok(vec![10, 20, 30, 40]),
    };

    assert_struct!(
        response,
        Response {
            data: Ok([10, 20, ..]),
        }
    );

    assert_struct!(response, Response { data: Ok([.., 40]) });
}

#[test]
fn test_result_vec_err() {
    let response = Response {
        data: Err("error".to_string()),
    };

    assert_struct!(response, Response { data: Err("error") });
}

// Test nested slices (Vec<Vec<T>>)
#[derive(Debug)]
struct Matrix {
    rows: Vec<Vec<i32>>,
}

#[test]
fn test_nested_slices() {
    let matrix = Matrix {
        rows: vec![vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]],
    };

    assert_struct!(
        matrix,
        Matrix {
            rows: [[1, 2, 3], [4, 5, 6], [7, 8, 9],],
        }
    );
}

#[test]
fn test_nested_slices_partial() {
    let matrix = Matrix {
        rows: vec![vec![1, 2, 3, 4], vec![5, 6, 7, 8], vec![9, 10, 11, 12]],
    };

    // Partial matching on outer slice
    assert_struct!(
        matrix,
        Matrix {
            rows: [[1, 2, 3, 4], [5, 6, 7, 8], ..,],
        }
    );

    // Partial matching on inner slices
    assert_struct!(
        matrix,
        Matrix {
            rows: [[1, 2, ..], [5, 6, ..], [9, 10, ..],],
        }
    );

    // Mixed partial matching
    assert_struct!(
        matrix,
        Matrix {
            rows: [[1, .., 4], ..,],
        }
    );
}

#[test]
fn test_nested_slices_with_comparisons() {
    let matrix = Matrix {
        rows: vec![vec![10, 20, 30], vec![40, 50, 60]],
    };

    assert_struct!(matrix, Matrix {
        rows: [
            [> 5, < 25, == 30],
            [>= 40, <= 50, > 55],
        ],
    });
}

// Multiple .. patterns should cause a compilation error
// This test ensures that our macro correctly rejects multiple .. patterns
// by generating invalid Rust code that the compiler will catch
error_message_test!(
    "slices_errors/slice_multiple_rest_patterns_fails.rs",
    slice_multiple_rest_patterns_fails
);

error_message_test!("slices_errors/slice_pattern.rs", slice_pattern);
