#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Inner {
    value: i32,
}

#[derive(Debug)]
struct Outer {
    inner: Inner,
}

pub fn test_case() {
    let data = Outer {
        inner: Inner {
            value: 42,
        },
    };

    assert_struct!(data, Outer {
        inner.value: 100,  // Should be 42, will fail
        ..
    });
}