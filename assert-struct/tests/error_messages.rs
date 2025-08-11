// Test error messages with exact format checking
// Each test case is in its own module file for stable line numbers

use std::panic;

// Helper to capture panic message
fn capture_panic_message<F: FnOnce() + panic::UnwindSafe>(f: F) -> String {
    let result = panic::catch_unwind(f);
    let err = result.unwrap_err();
    err.downcast_ref::<String>()
        .map(|s| s.as_str())
        .or_else(|| err.downcast_ref::<&str>().copied())
        .unwrap()
        .to_string()
}

// Each test case is in a separate module file
#[path = "error_messages/simple_field_mismatch.rs"]
mod simple_field_mismatch;
#[path = "error_messages/nested_comparison.rs"]
mod nested_comparison;
#[path = "error_messages/equality_pattern.rs"]
mod equality_pattern;
#[path = "error_messages/range_pattern.rs"]
mod range_pattern;
#[path = "error_messages/enum_variant.rs"]
mod enum_variant;

#[test]
fn test_simple_field_mismatch() {
    let message = capture_panic_message(|| {
        simple_field_mismatch::test_case();
    });

    // Line 15 is where the assertion happens in simple_field_mismatch.rs
    let expected = r#"assert_struct! failed:

   | User {
value mismatch:
  --> `user.name` (line 15)
   |     name: "Bob",
   |           ^^^^^ actual: "Alice"
   | }"#;

    assert_eq!(message, expected);
}

#[test]
fn test_nested_comparison() {
    let message = capture_panic_message(|| {
        nested_comparison::test_case();
    });

    // Line 21 is where the assertion happens in nested_comparison.rs
    let expected = r#"assert_struct! failed:

   | User { ... Profile {
comparison mismatch:
  --> `user.profile.age` (line 21)
   |         age: >= 18,
   |              ^^^^^ actual: 17
   | }"#;

    assert_eq!(message, expected);
}

#[test]
fn test_equality_pattern() {
    let message = capture_panic_message(|| {
        equality_pattern::test_case();
    });

    // Line 12 is where the assertion happens in equality_pattern.rs
    let expected = r#"assert_struct! failed:

   | Config {
equality mismatch:
  --> `config.timeout` (line 12)
   |     timeout: == expected_timeout,
   |                 ^^^^^^^^^^^^^^^^ actual: 30
   |                                  expected: 60
   | }"#;

    assert_eq!(message, expected);
}

#[test]
fn test_range_pattern() {
    let message = capture_panic_message(|| {
        range_pattern::test_case();
    });

    let expected = r#"assert_struct! failed:

   | Person {
range mismatch:
  --> `person.age` (line 11)
   |     age: 18 ..= 65,
   |          ^^^^^^^^^ actual: 75
   | }"#;

    assert_eq!(message, expected);
}

#[test]
fn test_enum_variant_mismatch() {
    let message = capture_panic_message(|| {
        enum_variant::test_case();
    });

    // This currently uses simple format, but we're testing it anyway
    let expected = r#"assert_struct! failed:

enum variant mismatch:
  --> `value` (assert-struct/tests/error_messages/enum_variant.rs:6)
  actual: Some
  expected: None"#;

    assert_eq!(message, expected);
}