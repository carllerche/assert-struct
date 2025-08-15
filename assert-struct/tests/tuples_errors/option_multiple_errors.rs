#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Container {
    id: u32,
    value: Option<(u32, String, bool)>,
}

pub fn test_case() {
    let container = Container {
        id: 4,
        value: Some((75, "test".to_string(), false)),
    };

    assert_struct!(
        container,
        Container {
            id: 4,
            value: Some((100, "other".to_string(), true)),  // All three fields wrong
        }
    );
}