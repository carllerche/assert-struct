#![allow(dead_code)]
use assert_struct::assert_struct;

pub fn test_case() {
    let items = vec![1, 2, 3];
    assert_struct!(items, #(1, 2)); // too few patterns, no rest
}
