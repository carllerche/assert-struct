use assert_struct::assert_struct;

#[macro_use]
mod util;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
    email: String,
    score: f64,
}

// Snapshot tests for error messages
error_message_test!(
    "multiple_failures_errors/multiple_field_failures.rs",
    multiple_field_failures
);
error_message_test!(
    "multiple_failures_errors/nested_multiple_failures.rs",
    nested_multiple_failures
);
error_message_test!(
    "multiple_failures_errors/option_multiple_failures.rs",
    option_multiple_failures
);
error_message_test!(
    "multiple_failures_errors/single_failure_unchanged.rs",
    single_failure_unchanged
);
error_message_test!(
    "multiple_failures_errors/mixed_level_failures.rs",
    mixed_level_failures
);

// Test that all passing assertions still work
#[test]
fn test_all_passing_unchanged() {
    let user = User {
        name: "Diana".to_string(),
        age: 25,
        email: "diana@example.com".to_string(),
        score: 92.0,
    };

    // All assertions pass - should work as before
    assert_struct!(user, User {
        name: "Diana",
        age: > 20,
        email: "diana@example.com",
        score: > 90.0,
    });
}
