use assert_struct::assert_struct;
use std::collections::HashMap;

#[derive(Debug)]
struct TestData {
    string_map: HashMap<String, String>,
}

pub fn test_case() {
    let mut string_map = HashMap::new();
    string_map.insert("key1".to_string(), "actual_value".to_string());
    
    let data = TestData {
        string_map,
    };

    // This should fail because value doesn't match
    assert_struct!(data, TestData {
        string_map: #{ "key1": "expected_value", .. },
    });
}