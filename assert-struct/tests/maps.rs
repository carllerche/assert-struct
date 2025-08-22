use assert_struct::assert_struct;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug)]
struct TestData {
    string_map: HashMap<String, String>,
    int_map: HashMap<String, i32>,
    btree_map: BTreeMap<String, String>,
    nested_map: HashMap<String, TestNested>,
}

#[derive(Debug)]
struct TestNested {
    value: String,
    count: i32,
}

#[test]
fn test_exact_map_matching() {
    let mut string_map = HashMap::new();
    string_map.insert("key1".to_string(), "value1".to_string());

    let data = TestData {
        string_map,
        int_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        nested_map: HashMap::new(),
    };

    // Test exact matching with single entry
    assert_struct!(data, TestData {
        string_map: #{ "key1": "value1" },
        ..
    });
}

#[test]
fn test_partial_map_matching() {
    let mut string_map = HashMap::new();
    string_map.insert("key1".to_string(), "value1".to_string());
    string_map.insert("key2".to_string(), "value2".to_string());
    string_map.insert("key3".to_string(), "value3".to_string());

    let data = TestData {
        string_map,
        int_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        nested_map: HashMap::new(),
    };

    // Test partial matching - only check some keys
    assert_struct!(data, TestData {
        string_map: #{ "key1": "value1", "key3": "value3", .. },
        ..
    });
}

#[test]
fn test_map_with_comparison_operators() {
    let mut int_map = HashMap::new();
    int_map.insert("count".to_string(), 42);
    int_map.insert("score".to_string(), 95);
    int_map.insert("level".to_string(), 3);

    let data = TestData {
        string_map: HashMap::new(),
        int_map,
        btree_map: BTreeMap::new(),
        nested_map: HashMap::new(),
    };

    // Test with comparison operators
    assert_struct!(data, TestData {
        int_map: #{
            "count": > 40,
            "score": >= 90,
            "level": <= 5,
            ..
        },
        ..
    });
}

#[test]
fn test_map_with_equality_operators() {
    let mut int_map = HashMap::new();
    int_map.insert("status".to_string(), 200);
    int_map.insert("error_code".to_string(), 0);

    let data = TestData {
        string_map: HashMap::new(),
        int_map,
        btree_map: BTreeMap::new(),
        nested_map: HashMap::new(),
    };

    // Test with equality operators
    assert_struct!(data, TestData {
        int_map: #{
            "status": == 200,
            "error_code": != 1,
            ..
        },
        ..
    });
}

#[test]
fn test_btree_map() {
    let mut btree_map = BTreeMap::new();
    btree_map.insert("first".to_string(), "alpha".to_string());
    btree_map.insert("second".to_string(), "beta".to_string());

    let data = TestData {
        string_map: HashMap::new(),
        int_map: HashMap::new(),
        btree_map,
        nested_map: HashMap::new(),
    };

    // Test BTreeMap duck typing
    assert_struct!(data, TestData {
        btree_map: #{ "first": "alpha", "second": "beta" },
        ..
    });
}

#[test]
fn test_nested_patterns() {
    let mut nested_map = HashMap::new();
    nested_map.insert(
        "item1".to_string(),
        TestNested {
            value: "test".to_string(),
            count: 5,
        },
    );
    nested_map.insert(
        "item2".to_string(),
        TestNested {
            value: "example".to_string(),
            count: 10,
        },
    );

    let data = TestData {
        string_map: HashMap::new(),
        int_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        nested_map,
    };

    // Test nested struct patterns in map values
    assert_struct!(data, TestData {
        nested_map: #{
            "item1": TestNested {
                value: "test",
                count: >= 5,
                ..
            },
            "item2": TestNested {
                value: "example",
                count: > 5,
                ..
            },
        },
        ..
    });
}

#[test]
#[cfg(feature = "regex")]
fn test_map_with_regex_patterns() {
    let mut string_map = HashMap::new();
    string_map.insert("email".to_string(), "user@example.com".to_string());
    string_map.insert("phone".to_string(), "123-456-7890".to_string());

    let data = TestData {
        string_map,
        int_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        nested_map: HashMap::new(),
    };

    // Test with regex patterns
    assert_struct!(data, TestData {
        string_map: #{
            "email": =~ r".*@.*\.com",
            "phone": =~ r"\d{3}-\d{3}-\d{4}",
            ..
        },
        ..
    });
}

#[test]
#[should_panic]
fn test_exact_length_mismatch() {
    let mut string_map = HashMap::new();
    string_map.insert("key1".to_string(), "value1".to_string());
    string_map.insert("key2".to_string(), "value2".to_string());

    let data = TestData {
        string_map,
        int_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        nested_map: HashMap::new(),
    };

    // This should fail because we have 2 entries but pattern expects exactly 1
    assert_struct!(data, TestData {
        string_map: #{ "key1": "value1" },
        ..
    });
}

#[test]
#[should_panic]
fn test_missing_key() {
    let mut string_map = HashMap::new();
    string_map.insert("key1".to_string(), "value1".to_string());

    let data = TestData {
        string_map,
        int_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        nested_map: HashMap::new(),
    };

    // This should fail because "missing_key" doesn't exist
    assert_struct!(data, TestData {
        string_map: #{ "missing_key": "value", .. },
        ..
    });
}

#[test]
#[should_panic]
fn test_value_mismatch() {
    let mut string_map = HashMap::new();
    string_map.insert("key1".to_string(), "actual_value".to_string());

    let data = TestData {
        string_map,
        int_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        nested_map: HashMap::new(),
    };

    // This should fail because value doesn't match
    assert_struct!(data, TestData {
        string_map: #{ "key1": "expected_value", .. },
        ..
    });
}

#[test]
fn test_empty_map() {
    let data = TestData {
        string_map: HashMap::new(),
        int_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        nested_map: HashMap::new(),
    };

    // Test empty map matching
    assert_struct!(data, TestData {
        string_map: #{},
        ..
    });
}

#[test]
fn test_mixed_patterns() {
    let mut int_map = HashMap::new();
    int_map.insert("exact".to_string(), 42);
    int_map.insert("greater".to_string(), 100);
    int_map.insert("range_val".to_string(), 50);

    let data = TestData {
        string_map: HashMap::new(),
        int_map,
        btree_map: BTreeMap::new(),
        nested_map: HashMap::new(),
    };

    // Test mixing different pattern types
    assert_struct!(data, TestData {
        int_map: #{
            "exact": 42,           // Simple pattern
            "greater": > 50,       // Comparison pattern
            "range_val": 25..75,   // Range pattern
            ..
        },
        ..
    });
}
