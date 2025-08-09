use assert_struct::assert_struct;

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
#[should_panic(expected = "pattern mismatch")]
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

#[test]
#[should_panic]
fn test_slice_element_mismatch() {
    let container = Container {
        items: vec![1, 2, 3],
        names: vec!["a".to_string(), "b".to_string()],
        data: vec![10, 20],
    };

    assert_struct!(
        container,
        Container {
            items: [1, 2, 4],  // Last element wrong
            names: ["a", "c"], // Second element wrong
            data: [10, 20],
        }
    );
}

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

    // TODO: Support partial matching on nested structs in slices
    // assert_struct!(inventory, Inventory {
    //     items: [
    //         Item { id: 1, .. },  // Only check id
    //         Item { value: "shield", .. },  // Only check value
    //         Item { id: > 0, value: "potion" },  // Check both with comparison
    //     ],
    // });

    // For now, test basic nested struct matching
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
                Item {
                    id: 3,
                    value: "potion".to_string()
                },
            ],
        }
    );
}

// Test with Option<Vec<T>>
#[derive(Debug)]
struct Config {
    values: Option<Vec<u32>>,
}

// TODO: Support slice syntax inside Option
// #[test]
// fn test_option_vec_some() {
//     let config = Config {
//         values: Some(vec![1, 2, 3]),
//     };
//
//     assert_struct!(config, Config {
//         values: Some([1, 2, 3]),
//     });
// }

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
