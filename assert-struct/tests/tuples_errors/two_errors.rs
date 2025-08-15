#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct TupleHolder {
    id: u32,
    pair: (u32, String),
}

pub fn test_case() {
    let holder = TupleHolder {
        id: 1,
        pair: (50, "actual".to_string()),
    };

    assert_struct!(
        holder,
        TupleHolder {
            id: 1,
            pair: (100, "expected"),  // Both fields wrong
        }
    );
}