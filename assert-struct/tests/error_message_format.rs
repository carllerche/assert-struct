use assert_struct::assert_struct;
use std::panic;

// Helper function to capture panic message
fn get_panic_message<F: FnOnce() + panic::UnwindSafe>(f: F) -> String {
    let result = panic::catch_unwind(f);
    match result {
        Err(err) => {
            if let Some(s) = err.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = err.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "Unknown panic message".to_string()
            }
        }
        Ok(_) => panic!("Expected panic but none occurred"),
    }
}

#[test]
fn test_basic_field_mismatch_format() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct User {
        name: String,
        age: u32,
    }

    let user = User {
        name: "Alice".to_string(),
        age: 18,
    };

    let message = get_panic_message(|| {
        assert_struct!(user, User { name: "Bob", .. });
    });

    // Check the fancy format with inline pattern
    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("   | User {"));
    assert!(message.contains("value mismatch:"));
    assert!(message.contains("--> `user.name`"));
    assert!(message.contains("   |     name: \"Bob\","));
    assert!(message.contains("^^^^^ actual: \"Alice\""));
    assert!(message.contains("   |     .."));
    assert!(message.contains("   | }"));
}

#[test]
fn test_comparison_pattern_format() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct User {
        name: String,
        age: u32,
    }

    let user = User {
        name: "Alice".to_string(),
        age: 25,
    };

    let message = get_panic_message(|| {
        assert_struct!(user, User {
            age: > 30,
            ..
        });
    });

    // Check the fancy format
    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("   | User {"));
    assert!(message.contains("comparison mismatch:"));
    assert!(message.contains("--> `user.age`"));
    assert!(message.contains("   |     age: > 30,"));
    assert!(message.contains("actual: 25"));
    assert!(message.contains("   |     .."));
    assert!(message.contains("   | }"));
}

#[test]
fn test_nested_field_path_format() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Response {
        user: User,
        status: Status,
    }

    #[derive(Debug)]
    #[allow(dead_code)]
    struct User {
        profile: Profile,
    }

    #[derive(Debug)]
    #[allow(dead_code)]
    struct Profile {
        age: u32,
        verified: bool,
    }

    #[derive(Debug)]
    #[allow(dead_code)]
    struct Status {
        code: i32,
    }

    let response = Response {
        user: User {
            profile: Profile {
                age: 25,
                verified: false,
            },
        },
        status: Status { code: 200 },
    };

    let message = get_panic_message(|| {
        assert_struct!(response, Response {
            user: User {
                profile: Profile {
                    age: > 30,
                    ..
                },
                ..
            },
            ..
        });
    });

    // Check the fancy format for nested structures
    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("   | Response { ... Profile {"));
    assert!(message.contains("comparison mismatch:"));
    assert!(message.contains("--> `response.user.profile.age`"));
    assert!(message.contains("   |             age: > 30,"));
    assert!(message.contains("actual: 25"));
    assert!(message.contains("   |             .."));
    assert!(message.contains("   | }"));
}

#[test]
fn test_range_pattern_format() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct User {
        age: u32,
    }

    let user = User { age: 17 };

    let message = get_panic_message(|| {
        assert_struct!(user, User { age: 18..=65 });
    });

    // Check the fancy format
    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("   | User {"));
    assert!(message.contains("range mismatch:"));
    assert!(message.contains("--> `user.age`"));
    assert!(message.contains("   |     age: 18 ..= 65,"));
    assert!(message.contains("actual: 17"));
    assert!(message.contains("   | }"));
}

#[test]
fn test_enum_variant_mismatch_format() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct User {
        age: Option<u32>,
    }

    let user = User { age: None };

    let message = get_panic_message(|| {
        assert_struct!(user, User { age: Some(30) });
    });

    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("enum variant mismatch:"));
    assert!(message.contains("--> `user.age`"));
    assert!(message.contains("actual: None"));
    assert!(message.contains("expected: Some"));

    println!("Actual error message:\n{}", message);
}

#[cfg(feature = "regex")]
#[test]
fn test_regex_pattern_format() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct User {
        email: String,
    }

    let user = User {
        email: "alice@wrong.com".to_string(),
    };

    let message = get_panic_message(|| {
        assert_struct!(user, User {
            email: =~ r"@example\.com$",
        });
    });

    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("regex pattern mismatch:"));
    assert!(message.contains("--> `user.email`"));
    assert!(message.contains("actual: \"alice@wrong.com\""));
    assert!(message.contains("expected: =~ r\"@example\\.com$\""));

    println!("Actual error message:\n{}", message);
}

#[test]
fn test_slice_pattern_format() {
    #[derive(Debug)]
    struct Data {
        values: Vec<i32>,
    }

    let data = Data {
        values: vec![1, 2, 3],
    };

    let message = get_panic_message(|| {
        assert_struct!(data, Data { values: [1, 5, 3] });
    });

    // Check the format - slices don't use fancy format yet since elements fail individually
    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("value mismatch:"));
    assert!(message.contains("--> `data.values.[1]`"));
    assert!(message.contains("actual: 2"));
    assert!(message.contains("expected: 5"));
}
