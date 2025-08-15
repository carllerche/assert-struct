#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct User {
    email: String,
}

pub fn test_case() {
    let user = User {
        email: "alice@wrong.com".to_string(),
    };

    assert_struct!(user, User {
        email: =~ r"@example\.com$",
    });
}