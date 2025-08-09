use assert_struct::assert_struct;

#[derive(Debug)]
struct Container {
    items: Vec<u32>,
}

fn main() {
    let container = Container {
        items: vec![1, 2, 3, 4, 5],
    };
    
    // This should fail to compile because Rust doesn't allow multiple .. in slice patterns
    assert_struct!(container, Container {
        items: [1, .., 3, .., 5],
    });
}