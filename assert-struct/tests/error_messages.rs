// Test error messages with exact format checking
// Each test case is in its own module file for stable line numbers

#[macro_use]
mod util;

error_message_test!(
    "error_messages/simple_field_mismatch.rs",
    simple_field_mismatch,
    r#"assert_struct! failed:

   | User {
value mismatch:
  --> `user.name` (line 15)
   |     name: "Bob",
   |           ^^^^^ actual: "Alice"
   | }"#
);

error_message_test!(
    "error_messages/nested_comparison.rs",
    nested_comparison,
    r#"assert_struct! failed:

   | User { ... Profile {
comparison mismatch:
  --> `user.profile.age` (line 21)
   |         age: >= 18,
   |              ^^^^^ actual: 17
   | }"#
);

error_message_test!(
    "error_messages/equality_pattern.rs",
    equality_pattern,
    r#"assert_struct! failed:

   | Config {
equality mismatch:
  --> `config.timeout` (line 12)
   |     timeout: == expected_timeout,
   |                 ^^^^^^^^^^^^^^^^ actual: 30
   |                                  expected: 60
   | }"#
);

error_message_test!(
    "error_messages/range_pattern.rs",
    range_pattern,
    r#"assert_struct! failed:

   | Person {
range mismatch:
  --> `person.age` (line 11)
   |     age: 18 ..= 65,
   |          ^^^^^^^^^ actual: 75
   | }"#
);

error_message_test!(
    "error_messages/enum_variant.rs",
    enum_variant,
    r#"assert_struct! failed:

enum variant mismatch:
  --> `value` (assert-struct/tests/error_messages/enum_variant.rs:6)
  actual: Some
  expected: None"#
);

error_message_test!(
    "error_messages/comparison_pattern.rs",
    comparison_pattern,
    r#"assert_struct! failed:

   | User {
comparison mismatch:
  --> `user.age` (line 15)
   |     age: > 30,
   |          ^^^^ actual: 25
   |     ..
   | }"#
);

error_message_test!(
    #[cfg(feature = "regex")]
    "error_messages/regex_pattern.rs",
    regex_pattern,
    r#"assert_struct! failed:

regex pattern mismatch:
  --> `user.email` (assert-struct/tests/error_messages/regex_pattern.rs:13)
  actual: "alice@wrong.com"
  expected: =~ r"@example\.com$""#
);

error_message_test!(
    "error_messages/slice_pattern.rs",
    slice_pattern,
    r#"assert_struct! failed:

value mismatch:
  --> `data.values.[1]` (assert-struct/tests/error_messages/slice_pattern.rs:13)
  actual: 2
  expected: 5"#
);

error_message_test!(
    "error_messages/option_with_comparison.rs",
    option_with_comparison,
    r#"assert_struct! failed:

comparison mismatch:
  --> `data.value.Some` (assert-struct/tests/error_messages/option_with_comparison.rs:11)
  actual: 25
  expected: > 30"#
);
