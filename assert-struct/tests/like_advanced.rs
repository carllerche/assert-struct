#[cfg(feature = "regex")]
use assert_struct::{assert_struct, Like, CaseInsensitive, Prefix, Suffix};

#[derive(Debug)]
struct TestData {
    text: String,
    code: String,
    name: String,
}

// Test Vec patterns (OR logic)
#[cfg(feature = "regex")]
#[test]
fn test_vec_patterns_or_logic() {
    let data = TestData {
        text: "hello@example.com".to_string(),
        code: "ABC123".to_string(),
        name: "Alice".to_string(),
    };
    
    // Multiple regex patterns - matches if ANY pattern matches
    let email_patterns = vec![
        r".*@example\.com",
        r".*@test\.com",
        r".*@demo\.com",
    ];
    
    assert_struct!(data, TestData {
        text: =~ email_patterns,  // Matches because it ends with @example.com
        ..
    });
    
    // Test with String patterns
    let code_patterns = vec![
        String::from(r"^ABC"),
        String::from(r"^XYZ"),
    ];
    
    assert_struct!(data, TestData {
        code: =~ code_patterns,  // Matches because it starts with ABC
        ..
    });
}

// Test Option patterns (wildcard behavior)
#[cfg(feature = "regex")]
#[test]
fn test_option_patterns() {
    let data = TestData {
        text: "anything".to_string(),
        code: "test123".to_string(),
        name: "Bob".to_string(),
    };
    
    // None acts as a wildcard - matches anything
    let wildcard: Option<&str> = None;
    assert_struct!(data, TestData {
        text: =~ wildcard,  // Matches anything
        ..
    });
    
    // Some with a pattern
    let pattern: Option<&str> = Some(r"test\d+");
    assert_struct!(data, TestData {
        code: =~ pattern,  // Matches the regex
        ..
    });
}

// Test tuple patterns (AND logic)
#[cfg(feature = "regex")]
#[test]
fn test_tuple_patterns_and_logic() {
    let text = "hello world".to_string();
    
    // Both patterns must match
    let patterns = (r"^hello", r"world$");
    assert!(text.like(&patterns));  // Starts with "hello" AND ends with "world"
    
    let patterns2 = (r"^hello", r"xyz$");
    assert!(!text.like(&patterns2));  // Starts with "hello" but doesn't end with "xyz"
}

// Test case-insensitive matching
#[cfg(feature = "regex")]
#[test]
fn test_case_insensitive() {
    let data = TestData {
        text: "Hello World".to_string(),
        code: "AbC123".to_string(),
        name: "ALICE".to_string(),
    };
    
    assert_struct!(data, TestData {
        text: =~ CaseInsensitive("hello world".to_string()),
        code: =~ CaseInsensitive("abc123".to_string()),
        name: =~ CaseInsensitive("alice".to_string()),
    });
}

// Test prefix matching
#[cfg(feature = "regex")]
#[test]
fn test_prefix_matching() {
    let data = TestData {
        text: "hello world".to_string(),
        code: "PREFIX_123".to_string(),
        name: "Mr. Smith".to_string(),
    };
    
    assert_struct!(data, TestData {
        text: =~ Prefix("hello".to_string()),
        code: =~ Prefix("PREFIX".to_string()),
        name: =~ Prefix("Mr.".to_string()),
    });
}

// Test suffix matching
#[cfg(feature = "regex")]
#[test]
fn test_suffix_matching() {
    let data = TestData {
        text: "user@example.com".to_string(),
        code: "test_SUFFIX".to_string(),
        name: "John Jr.".to_string(),
    };
    
    assert_struct!(data, TestData {
        text: =~ Suffix(".com".to_string()),
        code: =~ Suffix("SUFFIX".to_string()),
        name: =~ Suffix("Jr.".to_string()),
    });
}

// Test with boxed closures
#[cfg(feature = "regex")]
#[test]
fn test_boxed_closure_pattern() {
    let data = TestData {
        text: "test123".to_string(),
        code: "ABC".to_string(),
        name: "Alice".to_string(),
    };
    
    // Custom validation logic in a closure
    let length_check: Box<dyn Fn(&String) -> bool> = Box::new(|s| s.len() == 7);
    let uppercase_check: Box<dyn Fn(&String) -> bool> = Box::new(|s| s.chars().all(|c| c.is_uppercase()));
    
    assert!(data.text.like(&length_check));  // "test123" has 7 characters
    assert!(data.code.like(&uppercase_check));  // "ABC" is all uppercase
}

// Test failure cases
#[cfg(feature = "regex")]
#[test]
#[should_panic(expected = "Value does not match pattern")]
fn test_vec_pattern_no_match() {
    let data = TestData {
        text: "user@other.org".to_string(),
        code: "123".to_string(),
        name: "Bob".to_string(),
    };
    
    let patterns = vec![
        r".*@example\.com",
        r".*@test\.com",
    ];
    
    assert_struct!(data, TestData {
        text: =~ patterns,  // Should panic - doesn't match any pattern
        ..
    });
}

#[cfg(feature = "regex")]
#[test]
#[should_panic(expected = "Value does not match pattern")]
fn test_case_insensitive_failure() {
    let data = TestData {
        text: "Hello World".to_string(),
        code: "123".to_string(),
        name: "Alice".to_string(),
    };
    
    assert_struct!(data, TestData {
        text: =~ CaseInsensitive("goodbye world".to_string()),  // Should panic
        ..
    });
}

#[cfg(feature = "regex")]
#[test]
#[should_panic(expected = "Value does not match pattern")]
fn test_prefix_failure() {
    let data = TestData {
        text: "world".to_string(),
        code: "123".to_string(),
        name: "Smith".to_string(),
    };
    
    assert_struct!(data, TestData {
        text: =~ Prefix("hello".to_string()),  // Should panic - wrong prefix
        ..
    });
}

// Test combining different pattern types
#[cfg(feature = "regex")]
#[test]
fn test_mixed_pattern_types() {
    use regex::Regex;
    
    let data = TestData {
        text: "TEST@EXAMPLE.COM".to_string(),
        code: "prefix_middle_suffix".to_string(),
        name: "Dr. John Smith Jr.".to_string(),
    };
    
    // Mix different pattern types in one assertion
    assert_struct!(data, TestData {
        text: =~ CaseInsensitive("test@example.com".to_string()),
        code: =~ Prefix("prefix".to_string()),
        name: =~ Suffix("Jr.".to_string()),
    });
    
    // Use regex for more complex patterns
    let name_regex = Regex::new(r"^Dr\..*Jr\.$").unwrap();
    assert_struct!(data, TestData {
        name: =~ name_regex,
        ..
    });
}