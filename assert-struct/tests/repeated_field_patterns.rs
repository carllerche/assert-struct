#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
    active: bool,
}

#[derive(Debug)]  
struct Container {
    foo: InnerStruct,
    bar: String,
}

#[derive(Debug)]
struct InnerStruct {
    bar: i32,
    baz: i32,
}

#[test]
fn test_repeated_field_patterns() {
    let user = User {
        name: "Alice".to_string(),
        age: 25,
        active: true,
    };

    // Multiple constraints on the same field should all be checked
    assert_struct!(user, User { 
        age: >= 10, 
        age: <= 99,
        age: != 100,  // Additional constraint
        .. 
    });
}

#[test]
fn test_repeated_field_with_equality() {
    let user = User {
        name: "Bob".to_string(),
        age: 30,
        active: false,
    };

    // Mix comparison and equality patterns on same field
    assert_struct!(user, User { 
        age: >= 18,
        age: == 30,
        .. 
    });
}

#[test]
fn test_repeated_method_field_access_patterns() {
    let container = Container {
        foo: InnerStruct { bar: 1, baz: 2 },
        bar: "test".to_string(),
    };
    
    // Multiple access patterns on the same nested field
    assert_struct!(container, Container { 
        foo.bar: 1, 
        foo.baz: 2,
        foo.bar: > 0,  // Additional constraint on foo.bar
        ..
    });
}

#[test]
fn test_repeated_complex_field_access() {
    let container = Container {
        foo: InnerStruct { bar: 42, baz: 100 },
        bar: "complex".to_string(),
    };
    
    // Complex repeated patterns with different operations
    assert_struct!(container, Container { 
        foo.bar: >= 40,
        foo.bar: <= 50, 
        foo.baz: > 50,
        foo.baz: != 99,
        ..
    });
}

#[test]
fn test_mixed_repeated_patterns() {
    let user = User {
        name: "Charlie".to_string(),
        age: 35,
        active: true,
    };

    // Mix direct field access and repeated patterns
    assert_struct!(user, User { 
        name: "Charlie",
        age: >= 30,
        age: < 40,
        active: true,
        .. 
    });
}