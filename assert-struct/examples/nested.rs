#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Hello {
    world: String,
}

pub fn main() {
    let actual = Hello {
        world: "hello world".to_string(),
    };

    assert_struct!(actual, _ {
        world: "world",
        ..
    });
}
