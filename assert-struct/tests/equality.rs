#![allow(dead_code)]
use assert_struct::assert_struct;

#[macro_use]
mod util;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
    score: f64,
    active: bool,
}

#[test]
fn test_equality_operator() {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        score: 95.5,
        active: true,
    };

    assert_struct!(user, User {
        name: == "Alice",
        age: == 30,
        score: == 95.5,
        active: == true,
    });
}

#[test]
fn test_inequality_operator() {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        score: 95.5,
        active: true,
    };

    assert_struct!(user, User {
        name: != "Bob",
        age: != 25,
        score: != 0.0,
        active: != false,
    });
}

#[test]
fn test_mixed_operators() {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        score: 95.5,
        active: true,
    };

    assert_struct!(user, User {
        name: == "Alice",
        age: >= 18,
        score: > 90.0,
        active: != false,
    });
}

// Test with Option
#[derive(Debug)]
struct Profile {
    bio: Option<String>,
    age: Option<u32>,
}

#[test]
fn test_equality_in_option() {
    let profile = Profile {
        bio: Some("Developer".to_string()),
        age: Some(30),
    };

    assert_struct!(profile, Profile {
        bio: Some(== "Developer"),
        age: Some(!= 0),
    });
}

// Test with tuples
#[derive(Debug)]
struct Data {
    pair: (i32, String),
    triple: (bool, f64, u32),
}

#[test]
fn test_equality_in_tuples() {
    let data = Data {
        pair: (42, "test".to_string()),
        triple: (true, 3.5, 100),
    };

    assert_struct!(data, Data {
        pair: (== 42, != "other"),
        triple: (!= false, == 3.5, != 0),
    });
}

// Test failure cases
#[test]
#[should_panic(expected = "assert_struct! failed")]
fn test_equality_failure() {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        score: 95.5,
        active: true,
    };

    assert_struct!(user, User {
        name: == "Bob",  // Should fail
        ..
    });
}

#[test]
#[should_panic(expected = "assert_struct! failed")]
fn test_inequality_failure() {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
        score: 95.5,
        active: true,
    };

    assert_struct!(user, User {
        age: != 30,  // Should fail
        ..
    });
}

// Error message tests
error_message_test!("equality_errors/equality_pattern.rs", equality_pattern);

// ============================================================================
// Tests with Struct Literals (merged from equality_struct.rs)
// ============================================================================

#[derive(Debug, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct Shape {
    origin: Point,
    center: Point,
}

#[test]
fn test_equality_with_struct_literal() {
    let shape = Shape {
        origin: Point { x: 0, y: 0 },
        center: Point { x: 10, y: 20 },
    };

    // Test that we can use == with struct expressions
    assert_struct!(shape, Shape {
        origin: == Point { x: 0, y: 0 },
        center: == Point { x: 10, y: 20 },
    });
}

#[test]
fn test_inequality_with_struct_literal() {
    let shape = Shape {
        origin: Point { x: 0, y: 0 },
        center: Point { x: 10, y: 20 },
    };

    // Test that we can use != with struct expressions
    assert_struct!(shape, Shape {
        origin: != Point { x: 5, y: 5 },
        center: != Point { x: 0, y: 0 },
    });
}

#[test]
fn test_mixed_patterns_and_equality() {
    let shape = Shape {
        origin: Point { x: 0, y: 0 },
        center: Point { x: 10, y: 20 },
    };

    // Mix direct patterns with equality operators
    assert_struct!(shape, Shape {
        origin: Point { x: 0, y: 0 },  // Direct pattern matching
        center: == Point { x: 10, y: 20 },  // Equality operator with expression
    });
}

// Test with Option containing structs
#[derive(Debug)]
struct Container {
    point: Option<Point>,
}

#[test]
fn test_equality_in_option_with_struct() {
    let container = Container {
        point: Some(Point { x: 5, y: 10 }),
    };

    assert_struct!(container, Container {
        point: Some(== Point { x: 5, y: 10 }),
    });
}

// Test with nested struct equality
#[derive(Debug, PartialEq)]
struct Size {
    width: u32,
    height: u32,
}

#[derive(Debug, PartialEq)]
struct Window {
    title: String,
    size: Size,
}

#[derive(Debug)]
struct App {
    main_window: Window,
}

#[test]
fn test_nested_struct_equality() {
    let app = App {
        main_window: Window {
            title: "Main".to_string(),
            size: Size {
                width: 800,
                height: 600,
            },
        },
    };

    assert_struct!(app, App {
        main_window: == Window {
            title: "Main".to_string(),
            size: Size { width: 800, height: 600 },
        },
    });
}

// Test failure cases
#[test]
#[should_panic(expected = "assert_struct! failed")]
fn test_struct_equality_failure() {
    let shape = Shape {
        origin: Point { x: 0, y: 0 },
        center: Point { x: 10, y: 20 },
    };

    assert_struct!(shape, Shape {
        origin: == Point { x: 1, y: 1 },  // Should fail
        ..
    });
}

#[test]
#[should_panic(expected = "assert_struct! failed")]
fn test_struct_inequality_failure() {
    let shape = Shape {
        origin: Point { x: 0, y: 0 },
        center: Point { x: 10, y: 20 },
    };

    assert_struct!(shape, Shape {
        center: != Point { x: 10, y: 20 },  // Should fail - they are equal
        ..
    });
}

// Test with more complex expressions
#[test]
fn test_equality_with_method_call() {
    let shape = Shape {
        origin: Point { x: 0, y: 0 },
        center: Point { x: 10, y: 20 },
    };

    let expected_origin = Point { x: 0, y: 0 };

    assert_struct!(shape, Shape {
        origin: == expected_origin,  // Use a variable
        center: == Point { x: 10, y: 20 },
    });
}

// Test that direct pattern matching still works and is distinct from equality
#[test]
fn test_pattern_vs_equality_distinction() {
    #[derive(Debug)]
    struct Data {
        // This Location doesn't implement PartialEq
        location: Location,
        // This Point does implement PartialEq
        point: Point,
    }

    #[derive(Debug)]
    struct Location {
        x: i32,
        y: i32,
    }

    let data = Data {
        location: Location { x: 5, y: 10 },
        point: Point { x: 5, y: 10 },
    };

    assert_struct!(data, Data {
        // Pattern matching - doesn't need PartialEq
        location: Location { x: 5, y: 10 },
        // Equality check - needs PartialEq
        point: == Point { x: 5, y: 10 },
    });
}
