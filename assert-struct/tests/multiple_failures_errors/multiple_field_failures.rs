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
        name: "Alice".to_string(),
        age: 17,  // Will fail: expected > 18
        email: "alice@wrong.com".to_string(),  // Will fail: expected alice@example.com
        score: 95.5,
    };

    // This should collect both failures and report them together
    assert_struct!(user, User {
        name: "Alice",
        age: > 18,
        email: "alice@example.com",  // Will fail
        score: > 90.0,
    });
}