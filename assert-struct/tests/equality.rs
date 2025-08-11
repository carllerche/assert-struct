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
#[should_panic(expected = "equality mismatch")]
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
#[should_panic(expected = "comparison mismatch")]
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
error_message_test!(
    "equality_errors/equality_pattern.rs",
    equality_pattern
);
