// Test to understand current macro expansion
use assert_struct::assert_struct;

#[derive(Debug)]
struct Simple {
    value: i32,
}

#[test]
fn test_simple_expansion() {
    let s = Simple { value: 42 };
    
    // Simple case to see how the macro expands
    assert_struct!(s, Simple {
        value: 42,
    });
}