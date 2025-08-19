#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Data {
    matrix: Vec<Vec<i32>>,
}

pub fn test_case() {
    let data = Data {
        matrix: vec![vec![1, 2, 3], vec![4, 5, 6]],
    };

    assert_struct!(data, Data {
        matrix[0][1]: 5,  // Should be 2, will fail
        ..
    });
}