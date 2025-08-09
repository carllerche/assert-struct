#[cfg(feature = "regex")]
use assert_struct::{Like, assert_struct};
#[cfg(feature = "regex")]
use regex::Regex;

#[derive(Debug)]
struct TestData {
    email: String,
    phone: String,
    name: String,
}

// Test backward compatibility with string literals
#[cfg(feature = "regex")]
#[test]
fn test_backward_compat_string_literal() {
    let data = TestData {
        email: "user@example.com".to_string(),
        phone: "123-456-7890".to_string(),
        name: "John Doe".to_string(),
    };

    // Should still work with string literals (backward compatibility)
    assert_struct!(data, TestData {
        email: =~ r".*@example\.com",
        phone: =~ r"^\d{3}-\d{3}-\d{4}$",
        name: =~ r"John.*",
    });
}

// Test with pre-compiled regex
#[cfg(feature = "regex")]
#[test]
fn test_with_precompiled_regex() {
    let data = TestData {
        email: "admin@company.org".to_string(),
        phone: "555-123-4567".to_string(),
        name: "Alice Smith".to_string(),
    };

    let email_regex = Regex::new(r"^[^@]+@[^@]+\.[^@]+$").unwrap();
    let phone_regex = Regex::new(r"^\d{3}-\d{3}-\d{4}$").unwrap();

    assert_struct!(data, TestData {
        email: =~ email_regex,
        phone: =~ phone_regex,
        ..
    });
}

// Test with regex patterns from variables
#[cfg(feature = "regex")]
#[test]
fn test_with_pattern_variables() {
    let data = TestData {
        email: "test@example.com".to_string(),
        phone: "999-888-7777".to_string(),
        name: "Bob Johnson".to_string(),
    };

    let domain_pattern = r".*@example\.com";
    let name_pattern = String::from(r"^Bob");

    assert_struct!(data, TestData {
        email: =~ domain_pattern,
        name: =~ name_pattern,
        ..
    });
}

// Test with function that returns a pattern
#[cfg(feature = "regex")]
#[test]
fn test_with_pattern_from_function() {
    fn get_email_pattern() -> &'static str {
        r".*@example\.com"
    }

    fn get_phone_regex() -> Regex {
        Regex::new(r"^\d{3}-\d{3}-\d{4}$").unwrap()
    }

    let data = TestData {
        email: "support@example.com".to_string(),
        phone: "111-222-3333".to_string(),
        name: "Charlie".to_string(),
    };

    assert_struct!(data, TestData {
        email: =~ get_email_pattern(),
        phone: =~ get_phone_regex(),
        ..
    });
}

// Test custom Like implementation
#[cfg(feature = "regex")]
#[test]
fn test_custom_like_implementation() {
    #[derive(Debug)]
    struct DomainPattern {
        domain: String,
    }

    impl Like<DomainPattern> for String {
        fn like(&self, pattern: &DomainPattern) -> bool {
            self.ends_with(&format!("@{}", pattern.domain))
        }
    }

    let data = TestData {
        email: "user@example.com".to_string(),
        phone: "123-456-7890".to_string(),
        name: "Test User".to_string(),
    };

    let domain = DomainPattern {
        domain: "example.com".to_string(),
    };

    assert_struct!(data, TestData {
        email: =~ domain,
        ..
    });
}

// Test failure cases
#[cfg(feature = "regex")]
#[test]
#[should_panic(expected = "Value does not match pattern")]
fn test_like_pattern_mismatch() {
    let data = TestData {
        email: "user@other.com".to_string(),
        phone: "123-456-7890".to_string(),
        name: "John Doe".to_string(),
    };

    let pattern = r".*@example\.com";

    assert_struct!(data, TestData {
        email: =~ pattern,  // Should panic - wrong domain
        ..
    });
}

#[cfg(feature = "regex")]
#[test]
#[should_panic(expected = "Value does not match regex pattern")]
fn test_backward_compat_failure() {
    let data = TestData {
        email: "not-an-email".to_string(),
        phone: "123-456-7890".to_string(),
        name: "John Doe".to_string(),
    };

    assert_struct!(data, TestData {
        email: =~ r"^[^@]+@[^@]+\.[^@]+$",  // Should panic - invalid email
        ..
    });
}

// Test complex expressions
#[cfg(feature = "regex")]
#[test]
fn test_complex_pattern_expressions() {
    let data = TestData {
        email: "test@example.com".to_string(),
        phone: "555-123-4567".to_string(),
        name: "Test Name".to_string(),
    };

    let patterns = [r".*@example\.com", r".*@other\.com"];

    assert_struct!(data, TestData {
        email: =~ patterns[0],  // Using array indexing
        ..
    });
}

// Test with Option fields
#[derive(Debug)]
struct OptionalData {
    email: Option<String>,
    phone: Option<String>,
}

#[cfg(feature = "regex")]
#[test]
fn test_with_option_fields() {
    let data = OptionalData {
        email: Some("user@example.com".to_string()),
        phone: Some("123-456-7890".to_string()),
    };

    let email_pattern = r".*@example\.com";
    let phone_regex = Regex::new(r"^\d{3}-\d{3}-\d{4}$").unwrap();

    assert_struct!(data, OptionalData {
        email: Some(=~ email_pattern),
        phone: Some(=~ phone_regex),
    });
}

// Test inside nested structures
#[derive(Debug)]
struct User {
    profile: Profile,
}

#[derive(Debug)]
struct Profile {
    email: String,
    username: String,
}

#[cfg(feature = "regex")]
#[test]
fn test_nested_like_patterns() {
    let user = User {
        profile: Profile {
            email: "alice@example.com".to_string(),
            username: "alice_doe".to_string(),
        },
    };

    let email_regex = Regex::new(r".*@example\.com").unwrap();
    let username_pattern = r"^[a-z_]+$";

    assert_struct!(user, User {
        profile: Profile {
            email: =~ email_regex,
            username: =~ username_pattern,
        },
    });
}
