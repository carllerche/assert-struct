#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct TupleHolder {
    id: u32,
    pair: (u32, String),
}

pub fn test_case() {
    let holder = TupleHolder {
        id: 4,
        pair: (75, "test".to_string()),
    };

    assert_struct!(
        holder,
        TupleHolder {
            id: 4,
            pair: (> 100, "other"),  // Comparison fails, string fails
        }
    );
}