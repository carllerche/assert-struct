use assert_struct::assert_struct;

#[derive(Debug)]
struct Container {
    value: Box<String>,
}

fn main() {
    let container = Container {
        value: Box::new("test".to_string()),
    };

    assert_struct!(container, Container {
        *value: == 42,
        ..
    });
}