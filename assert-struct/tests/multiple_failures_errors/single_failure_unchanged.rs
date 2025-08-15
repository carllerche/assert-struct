#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
    email: String,
    score: f64,
}

pub fn test_case() {
    let user = User {
        name: "Charlie".to_string(),
        age: 25,
        email: "charlie@example.com".to_string(),
        score: 85.0,
    };

    // Only one failure - should maintain current error format
    assert_struct!(user, User {
        name: "Charlie",
        age: > 30,  // This will fail
        email: "charlie@example.com",
        score: > 80.0,
    });
}