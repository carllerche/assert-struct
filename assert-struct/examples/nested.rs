#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

pub fn main() {
    let actual = User {
        name: "Alice".to_string(),
        age: 20,
    };

    assert_struct!(actual, _ {
        name: "Bob",
        age: 21,
        ..
    });
}
