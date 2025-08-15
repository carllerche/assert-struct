#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct DataHolder {
    values: Vec<u32>,
    name: String,
}

pub fn test_case() {
    let data = DataHolder {
        values: vec![1, 2, 3],
        name: "test".to_string(),
    };

    assert_struct!(
        data,
        DataHolder {
            values: &[1, 2, 4], // Wrong last element
            name: "test",
        }
    );
}