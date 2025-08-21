use assert_struct::assert_struct;

#[derive(Debug)]
struct Simple {
    value: i32,
}

fn main() {
    let s = Simple { value: 42 };

    assert_struct!(s, _ {
        value: 42
    });
}