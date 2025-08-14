use assert_struct::assert_struct;

#[macro_use]
mod util;

#[derive(Debug)]
struct Person {
    #[allow(dead_code)]
    name: String,
    age: u32,
    score: f64,
    level: i32,
    grade: u8,
}

// Test inclusive ranges with ..=
#[test]
fn test_inclusive_range_integers() {
    let person = Person {
        name: "Alice".to_string(),
        age: 25,
        score: 85.5,
        level: 10,
        grade: 75,
    };

    assert_struct!(
        person,
        Person {
            age: 18..=65, // Inclusive range
            level: 0..=100,
            grade: 0..=100,
            ..
        }
    );
}

// Test exclusive ranges with ..
#[test]
fn test_exclusive_range_integers() {
    let person = Person {
        name: "Bob".to_string(),
        age: 30,
        score: 92.3,
        level: 50,
        grade: 85,
    };

    assert_struct!(
        person,
        Person {
            age: 0..100, // Exclusive upper bound (30 < 100)
            level: -100..100,
            grade: 0..=254,
            ..
        }
    );
}

// Test ranges with floating point
#[test]
fn test_range_floating_point() {
    let person = Person {
        name: "Charlie".to_string(),
        age: 25,
        score: 75.5,
        level: 10,
        grade: 80,
    };

    assert_struct!(
        person,
        Person {
            score: 0.0..100.0, // Exclusive range for float
            ..
        }
    );

    assert_struct!(
        person,
        Person {
            score: 0.0..=100.0, // Inclusive range for float
            ..
        }
    );
}

// Test range from (unbounded end)
#[test]
fn test_range_from() {
    let person = Person {
        name: "David".to_string(),
        age: 45,
        score: 88.0,
        level: 25,
        grade: 90,
    };

    assert_struct!(
        person,
        Person {
            age: 18..,    // 18 or older
            score: 0.0.., // Non-negative
            level: 0..,
            ..
        }
    );
}

// Test range to (unbounded start)
#[test]
fn test_range_to() {
    let person = Person {
        name: "Eve".to_string(),
        age: 16,
        score: 95.0,
        level: -5,
        grade: 99,
    };

    assert_struct!(
        person,
        Person {
            age: ..18,    // Less than 18
            level: ..0,   // Negative
            grade: ..100, // Less than 100
            ..
        }
    );
}

// Test range to inclusive
#[test]
fn test_range_to_inclusive() {
    let person = Person {
        name: "Frank".to_string(),
        age: 18,
        score: 100.0,
        level: 0,
        grade: 100,
    };

    assert_struct!(
        person,
        Person {
            age: ..=18,      // 18 or younger
            score: ..=100.0, // Up to and including 100
            level: ..=0,     // Zero or negative
            grade: ..=100,   // Up to and including 100
            ..
        }
    );
}

// Note: Full range (..) is not supported in match patterns

// Test combining ranges with other operators
#[derive(Debug)]
struct MixedData {
    a: i32,
    b: i32,
    c: i32,
    d: i32,
}

#[test]
fn test_mixed_range_and_operators() {
    let data = MixedData {
        a: 25,
        b: 50,
        c: 75,
        d: 100,
    };

    assert_struct!(data, MixedData {
        a: 20..30,          // Range
        b: == 50,           // Exact equality
        c: > 70,            // Comparison
        d: >= 100,          // Comparison
    });
}

// Test failure cases
#[test]
#[should_panic(expected = "mismatch")]
fn test_range_failure_below() {
    let person = Person {
        name: "Too Young".to_string(),
        age: 17,
        score: 50.0,
        level: 0,
        grade: 50,
    };

    assert_struct!(
        person,
        Person {
            age: 18..=65, // Should fail: 17 is below range
            ..
        }
    );
}

#[test]
#[should_panic(expected = "mismatch")]
fn test_range_failure_above() {
    let person = Person {
        name: "Too Old".to_string(),
        age: 66,
        score: 50.0,
        level: 0,
        grade: 50,
    };

    assert_struct!(
        person,
        Person {
            age: 18..=65, // Should fail: 66 is above range
            ..
        }
    );
}

#[test]
#[should_panic(expected = "mismatch")]
fn test_range_exclusive_boundary_failure() {
    let person = Person {
        name: "Boundary".to_string(),
        age: 25,
        score: 100.0,
        level: 0,
        grade: 50,
    };

    assert_struct!(
        person,
        Person {
            score: 0.0..100.0, // Should fail: 100.0 is not less than 100.0 (exclusive)
            ..
        }
    );
}

// Test char ranges
#[derive(Debug)]
struct TextData {
    grade: char,
    category: char,
}

#[test]
fn test_char_range() {
    let data = TextData {
        grade: 'B',
        category: 'M',
    };

    assert_struct!(
        data,
        TextData {
            grade: 'A'..='F',    // Letter grades
            category: 'A'..='Z', // Uppercase letters
        }
    );
}

error_message_test!("ranges_errors/range_pattern.rs", range_pattern);
