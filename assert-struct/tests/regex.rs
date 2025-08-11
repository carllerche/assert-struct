#![cfg(feature = "regex")]

use assert_struct::assert_struct;

#[macro_use]
mod util;

#[derive(Debug)]
struct Message {
    id: String,
    content: String,
    email: String,
}

#[test]
fn test_regex_match() {
    let msg = Message {
        id: "user-12345".to_string(),
        content: "Hello, World!".to_string(),
        email: "alice@example.com".to_string(),
    };

    assert_struct!(
        msg,
        Message {
            id: =~ r"^user-\d+$",
            content: =~ r"Hello.*",
            email: =~ r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
        }
    );
}

#[test]
fn test_regex_partial_match() {
    let msg = Message {
        id: "msg-abc-123".to_string(),
        content: "Test message with numbers 42".to_string(),
        email: "test@test.org".to_string(),
    };

    assert_struct!(
        msg,
        Message {
            content: =~ r"\d+", // Contains digits
            ..
        }
    );
}

#[test]
#[should_panic(expected = "regex pattern mismatch")]
fn test_regex_mismatch() {
    let msg = Message {
        id: "invalid".to_string(),
        content: "Test".to_string(),
        email: "not-an-email".to_string(),
    };

    assert_struct!(
        msg,
        Message {
            id: "invalid",
            content: "Test",
            email: =~ r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$",
        }
    );
}

#[test]
fn test_mixed_matchers() {
    let msg = Message {
        id: "prefix-789".to_string(),
        content: "Mixed test".to_string(),
        email: "bob@example.com".to_string(),
    };

    assert_struct!(
        msg,
        Message {
            id: =~ r"^prefix-\d+$",
            content: "Mixed test",       // Exact match
            email: =~ r"@example\.com$", // Ends with @example.com
        }
    );
}

error_message_test!(
    #[cfg(feature = "regex")]
    "regex_errors/regex_pattern.rs",
    regex_pattern
);
