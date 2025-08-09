#[cfg(feature = "regex")]
use assert_struct::Like;

#[cfg(feature = "regex")]
#[test]
fn test_string_like_str_pattern() {
    let text = String::from("hello123");
    assert!(text.like(&r"\w+\d+"));
    assert!(text.like(&r"hello\d{3}"));
    assert!(!text.like(&r"goodbye"));
}

#[cfg(feature = "regex")]
#[test]
fn test_string_like_string_pattern() {
    let text = String::from("test@example.com");
    let pattern = String::from(r".*@example\.com");
    assert!(text.like(&pattern));

    let pattern2 = String::from(r".*@other\.com");
    assert!(!text.like(&pattern2));
}

#[cfg(feature = "regex")]
#[test]
fn test_str_like_str_pattern() {
    let text = "hello world";
    assert!(text.like(&r"hello\s+world"));
    assert!(text.like(&r"^hello"));
    assert!(!text.like(&r"^world"));
}

#[cfg(feature = "regex")]
#[test]
fn test_str_like_string_pattern() {
    let text = "abc123xyz";
    let pattern = String::from(r"[a-z]+\d+[a-z]+");
    assert!(text.like(&pattern));
}

#[cfg(feature = "regex")]
#[test]
fn test_like_with_precompiled_regex() {
    use regex::Regex;

    let re = Regex::new(r"^\d{3}-\d{3}-\d{4}$").unwrap();
    let phone = "123-456-7890";
    assert!(phone.like(&re));

    let invalid_phone = "123456789";
    assert!(!invalid_phone.like(&re));

    // Also test with String
    let phone_string = String::from("555-123-4567");
    assert!(phone_string.like(&re));
}

#[cfg(feature = "regex")]
#[test]
fn test_like_email_pattern() {
    let email = "user@example.com";
    assert!(email.like(&r"^[^@]+@[^@]+\.[^@]+$"));

    let email_string = String::from("admin@company.org");
    assert!(email_string.like(&r"^[^@]+@[^@]+\.[^@]+$"));

    let invalid_email = "not-an-email";
    assert!(!invalid_email.like(&r"^[^@]+@[^@]+\.[^@]+$"));
}

#[cfg(feature = "regex")]
#[test]
fn test_like_invalid_regex_returns_false() {
    let text = "test";
    // Invalid regex should return false rather than panic
    assert!(!text.like(&r"["));

    let text_string = String::from("test");
    assert!(!text_string.like(&r"["));
}

// Test custom implementation
#[cfg(feature = "regex")]
#[test]
fn test_custom_like_implementation() {
    struct EmailAddress(String);

    struct DomainPattern {
        domain: String,
    }

    impl Like<DomainPattern> for EmailAddress {
        fn like(&self, pattern: &DomainPattern) -> bool {
            self.0.ends_with(&format!("@{}", pattern.domain))
        }
    }

    let email = EmailAddress("user@example.com".to_string());
    let pattern = DomainPattern {
        domain: "example.com".to_string(),
    };
    assert!(email.like(&pattern));

    let wrong_pattern = DomainPattern {
        domain: "other.com".to_string(),
    };
    assert!(!email.like(&wrong_pattern));
}

// Test that trait is object-safe (can be used as dyn Like)
#[cfg(feature = "regex")]
#[test]
fn test_like_is_object_safe() {
    fn check_like<'a>(text: &dyn Like<&'a str>, pattern: &'a &str) -> bool {
        text.like(pattern)
    }

    let s = String::from("hello");
    assert!(check_like(&s, &r"h.*o"));
}
