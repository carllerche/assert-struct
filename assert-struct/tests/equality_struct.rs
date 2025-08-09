use assert_struct::assert_struct;

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
#[should_panic(expected = "Failed equality")]
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
#[should_panic(expected = "Failed inequality")]
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
        // This Point doesn't implement PartialEq
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
