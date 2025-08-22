use assert_struct::assert_struct;
use std::collections::HashMap;

#[derive(Debug)]
struct TestData {
    string_map: HashMap<String, String>,
}

pub fn test_case() {
    let mut string_map = HashMap::new();
    string_map.insert("key1".to_string(), "value1".to_string());
    string_map.insert("key2".to_string(), "value2".to_string());
    
    let data = TestData {
        string_map,
    };

    // This should fail because we have 2 entries but pattern expects exactly 1
    assert_struct!(data, TestData {
        string_map: #{ "key1": "value1" },
    });
}