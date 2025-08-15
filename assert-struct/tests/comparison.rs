#![allow(dead_code)]
use assert_struct::assert_struct;

#[macro_use]
mod util;

#[derive(Debug)]
struct Person {
    name: String,
    age: u32,
    height: f64,
    score: i32,
}

#[test]
fn test_greater_than() {
    let person = Person {
        name: "Alice".to_string(),
        age: 25,
        height: 5.6,
        score: 100,
    };

    assert_struct!(
        person,
        Person {
            age: > 18,
            height: > 5.0,
            score: > 50,
            ..
        }
    );
}

#[test]
fn test_greater_equal() {
    let person = Person {
        name: "Bob".to_string(),
        age: 18,
        height: 6.0,
        score: 0,
    };

    assert_struct!(
        person,
        Person {
            age: >= 18,
            height: >= 6.0,
            score: >= 0,
            ..
        }
    );
}

#[test]
fn test_less_than() {
    let person = Person {
        name: "Charlie".to_string(),
        age: 16,
        height: 5.2,
        score: -10,
    };

    assert_struct!(
        person,
        Person {
            age: < 18,
            height: < 6.0,
            score: < 0,
            ..
        }
    );
}

#[test]
fn test_less_equal() {
    let person = Person {
        name: "Diana".to_string(),
        age: 65,
        height: 5.5,
        score: 100,
    };

    assert_struct!(
        person,
        Person {
            age: <= 65,
            height: <= 5.5,
            score: <= 100,
            ..
        }
    );
}

#[test]
fn test_mixed_comparisons() {
    let person = Person {
        name: "Eve".to_string(),
        age: 30,
        height: 5.8,
        score: 75,
    };

    assert_struct!(
        person,
        Person {
            name: "Eve",
            age: >= 21,
            height: < 6.0,
            score: > 50,
        }
    );
}

#[test]
#[should_panic(expected = "mismatch")]
fn test_greater_than_failure() {
    let person = Person {
        name: "Frank".to_string(),
        age: 15,
        height: 5.0,
        score: 40,
    };

    assert_struct!(
        person,
        Person {
            age: > 18,  // This should fail
            ..
        }
    );
}

error_message_test!("comparison_errors/less_than_failure.rs", less_than_failure);

error_message_test!(
    "comparison_errors/comparison_pattern.rs",
    comparison_pattern
);

#[derive(Debug)]
struct Nested {
    inner: Person,
}

#[test]
fn test_nested_comparison() {
    let nested = Nested {
        inner: Person {
            name: "Helen".to_string(),
            age: 35,
            height: 5.7,
            score: 85,
        },
    };

    assert_struct!(
        nested,
        Nested {
            inner: Person {
                age: > 30,
                score: >= 80,
                ..
            }
        }
    );
}
