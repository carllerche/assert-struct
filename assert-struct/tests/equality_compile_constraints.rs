// This file tests that certain patterns correctly fail to compile

use assert_struct::assert_struct;

#[derive(Debug, PartialEq)]
struct Point {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct Shape {
    origin: Point,
}

#[test]
fn test_equality_requires_complete_expression() {
    let shape = Shape {
        origin: Point { x: 0, y: 0 },
    };

    // This SHOULD work - complete struct expression
    assert_struct!(shape, Shape {
        origin: == Point { x: 0, y: 0 },  // Complete expression
    });

    // The following would NOT compile if uncommented:
    // assert_struct!(shape, Shape {
    //     origin: == Point { .. },  // ERROR: `..` is not valid in expressions
    // });

    // But this DOES work - pattern matching without ==
    assert_struct!(
        shape,
        Shape {
            origin: Point { x: 0, .. }, // Pattern with partial matching
        }
    );
}

// This test demonstrates the key difference
#[test]
fn test_pattern_vs_expression_distinction() {
    #[derive(Debug, PartialEq)]
    struct Complex {
        a: i32,
        b: i32,
        c: i32,
    }

    #[derive(Debug)]
    struct Container {
        data: Complex,
    }

    let container = Container {
        data: Complex { a: 1, b: 2, c: 3 },
    };

    // Pattern matching - can use `..` to ignore fields
    assert_struct!(
        container,
        Container {
            data: Complex { a: 1, .. }, // Only check `a`, ignore b and c
        }
    );

    // Equality check - must specify complete value
    assert_struct!(container, Container {
        data: == Complex { a: 1, b: 2, c: 3 },  // Must specify all fields
    });

    // This would NOT compile:
    // assert_struct!(container, Container {
    //     data: == Complex { a: 1, .. },  // ERROR: Can't use .. in expression
    // });
}
