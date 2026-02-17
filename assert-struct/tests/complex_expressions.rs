#![allow(dead_code)]
use assert_struct::assert_struct;

// Helper functions for testing
fn get_min_age() -> u32 {
    18
}

fn compute_threshold() -> f64 {
    75.0
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[derive(Debug)]
struct Config {
    min_value: i32,
    max_value: i32,
    threshold: f64,
}

impl Config {
    fn get_limit(&self) -> i32 {
        self.max_value
    }
}

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
    score: f64,
    level: i32,
}

// Test function calls
#[test]
fn test_function_call_in_comparison() {
    let user = User {
        name: "Alice".to_string(),
        age: 25,
        score: 85.5,
        level: 10,
    };

    assert_struct!(user, User {
        age: >= get_min_age(),
        score: > compute_threshold(),
        ..
    });
}

// Test arithmetic expressions
#[test]
fn test_arithmetic_expression() {
    let user = User {
        name: "Bob".to_string(),
        age: 30,
        score: 95.0,
        level: 15,
    };

    assert_struct!(user, User {
        age: < 40 + 5,                    // Simple arithmetic
        score: >= 100.0 - 10.0,           // Float arithmetic
        level: == add(7, 8),              // Function call in arithmetic
        ..
    });
}

// Test field access
#[test]
fn test_field_access() {
    let config = Config {
        min_value: 10,
        max_value: 100,
        threshold: 50.0,
    };

    let user = User {
        name: "Charlie".to_string(),
        age: 25,
        score: 75.0,
        level: 50,
    };

    assert_struct!(user, User {
        level: >= config.min_value,
        score: > config.threshold,
        ..
    });

    // Also check max value separately
    assert_struct!(user, User {
        level: <= config.max_value,
        ..
    });
}

// Test method calls
#[test]
fn test_method_calls() {
    let config = Config {
        min_value: 10,
        max_value: 100,
        threshold: 50.0,
    };

    let user = User {
        name: "Dave".to_string(),
        age: 25,
        score: 75.0,
        level: 90,
    };

    assert_struct!(user, User {
        level: < config.get_limit(),
        name: == "Dave".to_string(),  // Method call on literal
        ..
    });
}

// Test complex nested expressions
#[test]
fn test_complex_nested_expression() {
    let config = Config {
        min_value: 10,
        max_value: 100,
        threshold: 50.0,
    };

    let user = User {
        name: "Eve".to_string(),
        age: 25,
        score: 75.0,
        level: 45,
    };

    assert_struct!(user, User {
        level: > config.min_value + 10,  // Field access + arithmetic
        score: < config.threshold * 2.0,  // Field access + multiplication
        age: == get_min_age() + 7,       // Function call + arithmetic
        ..
    });
}

// Test with closures
#[test]
fn test_closure_expression() {
    let threshold = 50;
    let compute = |x: i32| x * 2;

    let user = User {
        name: "Frank".to_string(),
        age: 25,
        score: 75.0,
        level: 110,
    };

    assert_struct!(user, User {
        level: > compute(threshold),
        ..
    });
}

// Test with array/vec indexing
#[test]
fn test_array_indexing() {
    let limits = [10, 20, 30, 40];
    let scores = [50.0, 60.0, 70.0, 80.0];

    let user = User {
        name: "Grace".to_string(),
        age: 25,
        score: 75.0,
        level: 35,
    };

    assert_struct!(user, User {
        level: > limits[2],
        score: < scores[3],
        ..
    });
}

// Test with Option values
#[derive(Debug)]
struct Settings {
    max_age: Option<u32>,
    min_score: Option<f64>,
}

#[test]
fn test_option_unwrap_in_expression() {
    let settings = Settings {
        max_age: Some(65),
        min_score: Some(70.0),
    };

    let user = User {
        name: "Helen".to_string(),
        age: 45,
        score: 75.0,
        level: 10,
    };

    assert_struct!(user, User {
        age: < settings.max_age.unwrap_or(100),
        score: >= settings.min_score.unwrap_or(0.0),
        ..
    });
}

// Test expressions in tuples
#[derive(Debug)]
struct Point {
    coords: (i32, i32),
}

#[test]
fn test_expressions_in_tuples() {
    let origin_x = 10;
    let origin_y = 20;

    let point = Point { coords: (15, 25) };

    assert_struct!(point, Point {
        coords: (> origin_x, > origin_y),
    });
}

// Test expressions in Option
#[derive(Debug)]
struct Container {
    value: Option<i32>,
}

#[test]
fn test_expression_in_option() {
    let min_value = 10;

    let container = Container { value: Some(20) };

    assert_struct!(container, Container {
        value: Some(> min_value),
    });
}

// Test with equality and complex expressions
#[test]
fn test_equality_with_complex_expression() {
    let base: i32 = 10;
    let multiplier: i32 = 3;

    let user = User {
        name: "Ivan".to_string(),
        age: 30,
        score: 75.0,
        level: 30,
    };

    assert_struct!(user, User {
        level: == base * multiplier,
        age: != (base + multiplier) as u32,
        ..
    });
}

// Test with string operations
#[test]
fn test_string_operations() {
    let prefix = "Hello";
    let suffix = "World";

    let user = User {
        name: "Hello, World".to_string(),
        age: 25,
        score: 75.0,
        level: 10,
    };

    assert_struct!(user, User {
        name: == format!("{}, {}", prefix, suffix),
        ..
    });
}

// Test chained method calls
#[test]
fn test_chained_method_calls() {
    let user = User {
        name: "john doe".to_string(),
        age: 25,
        score: 75.0,
        level: 10,
    };

    assert_struct!(user, User {
        name: == "JOHN DOE".to_lowercase(),
        ..
    });
}

// Test with const values
const MIN_AGE: u32 = 21;
const MAX_SCORE: f64 = 100.0;

#[test]
fn test_const_expressions() {
    let user = User {
        name: "Kate".to_string(),
        age: 25,
        score: 75.0,
        level: 10,
    };

    assert_struct!(user, User {
        age: >= MIN_AGE,
        score: <= MAX_SCORE,
        ..
    });
}

// Test macro expressions
macro_rules! double {
    ($x:expr) => {
        $x * 2
    };
}

#[test]
fn test_macro_in_expression() {
    let user = User {
        name: "Laura".to_string(),
        age: 30,
        score: 75.0,
        level: 20,
    };

    assert_struct!(user, User {
        level: == double!(10),
        ..
    });
}

// Test failure cases
#[test]
#[ignore = "error message format in flux"]
#[should_panic(expected = "assert_struct! failed")]
fn test_complex_expression_failure() {
    let user = User {
        name: "Mike".to_string(),
        age: 15,
        score: 75.0,
        level: 10,
    };

    assert_struct!(user, User {
        age: >= get_min_age() + 5,  // Should fail: 15 < 23
        ..
    });
}
