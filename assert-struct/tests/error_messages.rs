// Test error messages with exact format checking
// Each test case is in its own module file for stable line numbers

#[macro_use]
mod util;

error_message_test!(
    "error_messages/simple_field_mismatch.rs",
    simple_field_mismatch
);

error_message_test!(
    "error_messages/nested_comparison.rs",
    nested_comparison
);

error_message_test!(
    "error_messages/equality_pattern.rs",
    equality_pattern
);

error_message_test!(
    "error_messages/range_pattern.rs",
    range_pattern
);

error_message_test!(
    "error_messages/enum_variant.rs",
    enum_variant
);

error_message_test!(
    "error_messages/comparison_pattern.rs",
    comparison_pattern
);

error_message_test!(
    #[cfg(feature = "regex")]
    "error_messages/regex_pattern.rs",
    regex_pattern
);

error_message_test!(
    "error_messages/slice_pattern.rs",
    slice_pattern
);

error_message_test!(
    "error_messages/option_with_comparison.rs",
    option_with_comparison
);
