#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Data {
    score: i32,
}

#[derive(Debug)]
struct Container {
    data: Data,
}

pub fn test_case() {
    let container = Container {
        data: Data {
            score: 25,
        },
    };

    assert_struct!(container, Container {
        data.score: > 50,  // 25 is not > 50, will fail
        ..
    });
}