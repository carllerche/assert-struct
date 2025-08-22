use assert_struct::assert_struct;
use std::collections::HashMap;

#[derive(Debug)]
struct TestData {
    string_map: HashMap<String, String>,
}

pub fn test_case() {
    let mut string_map = HashMap::new();
    string_map.insert("key1".to_string(), "value1".to_string());
    
    let data = TestData {
        string_map,
    };

    // This should fail because "missing_key" doesn't exist
    assert_struct!(data, TestData {
        string_map: #{ "missing_key": "value", .. },
    });
}