use assert_struct::assert_struct;
use std::panic;

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
fn test_simple_value_mismatch_fancy() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct User {
        name: String,
        age: u32,
    }

    let user = User {
        name: "Alice".to_string(),
        age: 30,
    };

    let message = get_panic_message(|| {
        assert_struct!(
            user,
            User {
                name: "Bob",
                age: 30,
            }
        );
    });

    // Verify fancy format with inline pattern
    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("   | User {"));
    assert!(message.contains("value mismatch:"));
    assert!(message.contains("   |     name: \"Bob\","));
    assert!(message.contains("^^^^^ actual: \"Alice\""));
    assert!(message.contains("   | }"));
}

#[test]
fn test_comparison_operator_fancy() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Stats {
        score: u32,
        level: u32,
    }

    let stats = Stats {
        score: 50,
        level: 3,
    };

    let message = get_panic_message(|| {
        assert_struct!(stats, Stats {
            score: > 100,
            level: 3,
        });
    });

    // Verify comparison operator fancy format
    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("   | Stats {"));
    assert!(message.contains("comparison mismatch:"));
    assert!(message.contains("   |     score: > 100,"));
    assert!(message.contains("actual: 50"));
}

#[test]
fn test_range_pattern_fancy() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Person {
        age: u32,
        height: f32,
    }

    let person = Person {
        age: 17,
        height: 175.5,
    };

    let message = get_panic_message(|| {
        assert_struct!(
            person,
            Person {
                age: 18..=65,
                height: 175.5,
            }
        );
    });

    // Verify range pattern fancy format
    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("   | Person {"));
    assert!(message.contains("range mismatch:"));
    assert!(message.contains("   |     age: 18 ..= 65,"));
    assert!(message.contains("actual: 17"));
}

#[test]
fn test_nested_struct_fancy() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Response {
        status: u32,
        user: User,
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

    let response = Response {
        status: 200,
        user: User {
            profile: Profile {
                age: 25,
                verified: false,
            },
        },
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

    // Verify nested struct fancy format with abbreviated context
    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("   | Response { ... Profile {"));
    assert!(message.contains("comparison mismatch:"));
    assert!(message.contains("--> `response.user.profile.age`"));
    assert!(message.contains("   |             age: > 30,"));
    assert!(message.contains("actual: 25"));
}

#[test]
fn test_enum_variant_mismatch_fancy() {
    #[derive(Debug, PartialEq)]
    #[allow(dead_code)]
    enum Status {
        Active { since: u32 },
        Inactive,
        Pending,
    }

    let status = Status::Inactive;

    let message = get_panic_message(|| {
        assert_struct!(status, Status::Active { since: > 0 });
    });

    // Verify enum variant mismatch fancy format
    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("   | Status :: Active {"));
    assert!(message.contains("enum variant mismatch:"));
    assert!(message.contains("   | Status :: Active {"));
    assert!(message.contains("^^^^^^^^^^^^^^^^ actual: Inactive"));
}

#[test]
fn test_option_with_comparison_fancy() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Data {
        value: Option<u32>,
    }

    let data = Data { value: Some(25) };

    let message = get_panic_message(|| {
        assert_struct!(data, Data {
            value: Some(> 30),
        });
    });

    // Verify Option with comparison - currently doesn't use full fancy format for nested Some
    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("comparison mismatch:"));
    assert!(message.contains("--> `data.value.Some`"));
    assert!(message.contains("actual: 25"));
    assert!(message.contains("expected: > 30"));
}

#[test]
fn test_rest_pattern_fancy() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Config {
        name: String,
        enabled: bool,
        timeout: u32,
        retries: u32,
    }

    let config = Config {
        name: "test".to_string(),
        enabled: false,
        timeout: 30,
        retries: 3,
    };

    let message = get_panic_message(|| {
        assert_struct!(
            config,
            Config {
                name: "test",
                enabled: true,
                ..
            }
        );
    });

    // Verify rest pattern appears in output
    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("   | Config {"));
    assert!(message.contains("value mismatch:"));
    assert!(message.contains("   |     enabled: true,"));
    assert!(message.contains("actual: false"));
    assert!(message.contains("   |     .."));
    assert!(message.contains("   | }"));
}
