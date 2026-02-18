use assert_struct::assert_struct;
use std::collections::{BTreeMap, HashMap};

#[path = "util/mod.rs"]
mod util;

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

// Error message tests using insta snapshots
error_message_test!("map_errors/exact_length_mismatch.rs", exact_length_mismatch);
error_message_test!("map_errors/missing_key.rs", missing_key);
error_message_test!("map_errors/value_mismatch.rs", value_mismatch);

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

// Custom map type that only implements len() and get() to test duck typing
#[derive(Debug)]
struct CustomMap<K, V> {
    entries: Vec<(K, V)>,
}

impl<K, V> CustomMap<K, V> {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn insert(&mut self, key: K, value: V) {
        self.entries.push((key, value));
    }

    fn len(&self) -> usize {
        self.entries.len()
    }

    fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: std::borrow::Borrow<Q>,
        Q: Eq + ?Sized,
    {
        self.entries
            .iter()
            .find(|(k, _)| k.borrow() == key)
            .map(|(_, v)| v)
    }
}

#[derive(Debug)]
struct CustomMapData {
    string_map: CustomMap<String, String>,
    int_map: CustomMap<String, i32>,
}

#[test]
fn test_custom_map_duck_typing() {
    let mut string_map = CustomMap::new();
    string_map.insert("key1".to_string(), "value1".to_string());
    string_map.insert("key2".to_string(), "value2".to_string());

    let mut int_map = CustomMap::new();
    int_map.insert("count".to_string(), 42);

    let data = CustomMapData {
        string_map,
        int_map,
    };

    // Test exact matching with custom map
    assert_struct!(data, CustomMapData {
        string_map: #{ "key1": "value1", "key2": "value2" },
        ..
    });

    // Test partial matching with custom map
    assert_struct!(data, CustomMapData {
        string_map: #{ "key1": "value1", .. },
        int_map: #{ "count": 42 },
        ..
    });
}

#[test]
fn test_custom_map_with_patterns() {
    let mut int_map = CustomMap::new();
    int_map.insert("score".to_string(), 85);
    int_map.insert("level".to_string(), 3);

    let data = CustomMapData {
        string_map: CustomMap::new(),
        int_map,
    };

    // Test with comparison patterns on custom map
    assert_struct!(data, CustomMapData {
        int_map: #{
            "score": > 80,
            "level": <= 5,
            ..
        },
        ..
    });
}

#[test]
fn test_empty_custom_map() {
    let data = CustomMapData {
        string_map: CustomMap::new(),
        int_map: CustomMap::new(),
    };

    // Test empty custom map matching
    assert_struct!(data, CustomMapData {
        string_map: #{},
        int_map: #{},
        ..
    });
}

#[test]
#[allow(unused_variables)] // Wildcard patterns intentionally don't use field names
fn test_wildcard_only_pattern() {
    let mut string_map = HashMap::new();
    string_map.insert("any".to_string(), "value".to_string());
    string_map.insert("keys".to_string(), "here".to_string());

    let mut custom_map = CustomMap::new();
    custom_map.insert("custom".to_string(), "data".to_string());

    let data = TestData {
        string_map,
        int_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        nested_map: HashMap::new(),
    };

    let custom_data = CustomMapData {
        string_map: custom_map,
        int_map: CustomMap::new(),
    };

    // Test wildcard-only pattern - matches any map regardless of contents
    assert_struct!(data, TestData {
        string_map: #{ .. },  // Matches map with any contents
        int_map: #{ .. },     // Matches empty map too
        ..
    });

    assert_struct!(custom_data, CustomMapData {
        string_map: #{ .. },  // Matches custom map with contents
        int_map: #{ .. },     // Matches empty custom map
        ..
    });
}

#[test]
#[allow(unused_variables)] // Wildcard patterns intentionally don't use field names
fn test_empty_vs_wildcard_distinction() {
    let mut non_empty_map = HashMap::new();
    non_empty_map.insert("key".to_string(), "value".to_string());

    let empty_map = HashMap::new();

    let data_with_empty = TestData {
        string_map: empty_map,
        int_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        nested_map: HashMap::new(),
    };

    let data_with_content = TestData {
        string_map: non_empty_map,
        int_map: HashMap::new(),
        btree_map: BTreeMap::new(),
        nested_map: HashMap::new(),
    };

    // #{} requires exactly empty map (len() == 0)
    assert_struct!(data_with_empty, TestData {
        string_map: #{},
        ..
    });

    // #{ .. } matches any map regardless of content
    assert_struct!(data_with_content, TestData {
        string_map: #{ .. },
        ..
    });

    assert_struct!(data_with_empty, TestData {
        string_map: #{ .. },
        ..
    });
}

// Tests for issue #52: map patterns inside enum variants (Some, Ok, etc.)
#[test]
fn test_map_inside_some() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), "value".to_string());
    let optional_map: Option<HashMap<String, String>> = Some(map);

    assert_struct!(optional_map, Some(#{ "key": "value" }));
}

#[test]
fn test_map_inside_ok() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), 42);
    let result_map: Result<HashMap<String, i32>, String> = Ok(map);

    assert_struct!(result_map, Ok(#{ "key": 42 }));
}
