#![allow(dead_code)]
use assert_struct::assert_struct;

pub fn test_case() {
    // No element is < 0, so the pattern cannot be satisfied
    let items = vec![1, 2, 3];
    assert_struct!(items, #(1, < 0, 3));
}
