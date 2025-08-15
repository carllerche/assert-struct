#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct TupleHolder {
    id: u32,
    triple: (i32, String, bool),
}

pub fn test_case() {
    let holder = TupleHolder {
        id: 2,
        triple: (42, "actual".to_string(), true),
    };

    assert_struct!(
        holder,
        TupleHolder {
            id: 2,
            triple: (100, "expected", false),  // All three fields wrong
        }
    );
}