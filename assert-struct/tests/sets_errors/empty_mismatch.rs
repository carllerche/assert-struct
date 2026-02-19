#![allow(dead_code)]
use assert_struct::assert_struct;

pub fn test_case() {
    let items = vec![1];
    assert_struct!(items, #()); // expected empty but has 1 element
}
