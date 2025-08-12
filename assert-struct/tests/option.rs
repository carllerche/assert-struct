use assert_struct::assert_struct;

#[macro_use]
mod util;

// Basic Option field support tests.
// For advanced patterns (comparisons and regex inside Some), see option_advanced.rs

#[derive(Debug)]
struct User {
    id: u32,
    name: String,
    email: Option<String>,
    age: Option<u32>,
    verified_at: Option<i64>,
}

#[test]
fn test_some_exact_match() {
    let user = User {
        id: 1,
        name: "Alice".to_string(),
        email: Some("alice@example.com".to_string()),
        age: Some(30),
        verified_at: Some(1234567890),
    };

    assert_struct!(
        user,
        User {
            id: 1,
            name: "Alice",
            email: Some("alice@example.com"),
            age: Some(30),
            verified_at: Some(1234567890),
        }
    );
}

#[test]
fn test_none_match() {
    let user = User {
        id: 2,
        name: "Bob".to_string(),
        email: None,
        age: None,
        verified_at: None,
    };

    assert_struct!(
        user,
        User {
            id: 2,
            name: "Bob",
            email: None,
            age: None,
            verified_at: None,
        }
    );
}

#[test]
fn test_mixed_some_none() {
    let user = User {
        id: 3,
        name: "Charlie".to_string(),
        email: Some("charlie@test.com".to_string()),
        age: None,
        verified_at: Some(9876543210),
    };

    assert_struct!(
        user,
        User {
            id: 3,
            name: "Charlie",
            email: Some("charlie@test.com"),
            age: None,
            verified_at: Some(9876543210),
        }
    );
}

#[test]
fn test_partial_match_with_options() {
    let user = User {
        id: 4,
        name: "Diana".to_string(),
        email: Some("diana@example.com".to_string()),
        age: Some(25),
        verified_at: None,
    };

    // Only check some fields
    assert_struct!(
        user,
        User {
            name: "Diana",
            email: Some("diana@example.com"),
            ..
        }
    );
}

#[derive(Debug, PartialEq)]
struct Profile {
    bio: Option<String>,
    website: Option<String>,
    location: Option<Location>,
}

#[derive(Debug, PartialEq)]
struct Location {
    city: String,
    country: String,
}

#[test]
fn test_nested_option_struct() {
    let profile = Profile {
        bio: Some("Software developer".to_string()),
        website: Some("https://example.com".to_string()),
        location: Some(Location {
            city: "Boston".to_string(),
            country: "USA".to_string(),
        }),
    };

    assert_struct!(
        profile,
        Profile {
            bio: Some("Software developer"),
            website: Some("https://example.com"),
            location: Some(Location {
                city: "Boston".to_string(), // Nested structs still need .to_string()
                country: "USA".to_string(),
            }),
        }
    );
}

#[test]
fn test_nested_option_none() {
    let profile = Profile {
        bio: Some("Engineer".to_string()),
        website: None,
        location: None,
    };

    assert_struct!(
        profile,
        Profile {
            bio: Some("Engineer"),
            website: None,
            location: None,
        }
    );
}

#[derive(Debug)]
struct Settings {
    notifications: Option<Vec<String>>,
    max_items: Option<u32>,
    flags: Option<(bool, bool)>,
}

#[test]
fn test_option_with_collections() {
    let settings = Settings {
        notifications: Some(vec!["email".to_string(), "sms".to_string()]),
        max_items: Some(100),
        flags: Some((true, false)),
    };

    assert_struct!(
        settings,
        Settings {
            notifications: Some(vec!["email".to_string(), "sms".to_string()]),
            max_items: Some(100),
            flags: Some((true, false)),
        }
    );
}

#[test]
fn test_option_none_with_collections() {
    let settings = Settings {
        notifications: None,
        max_items: None,
        flags: None,
    };

    assert_struct!(
        settings,
        Settings {
            notifications: None,
            max_items: None,
            flags: None,
        }
    );
}

// Failure cases

#[test]
#[should_panic(expected = "enum variant mismatch")]
fn test_some_none_mismatch() {
    let user = User {
        id: 7,
        name: "Grace".to_string(),
        email: Some("grace@example.com".to_string()),
        age: None,
        verified_at: None,
    };

    assert_struct!(
        user,
        User {
            id: 7,
            name: "Grace",
            email: None,
            age: None,
            verified_at: None,
        }
    );
}

#[test]
#[should_panic(expected = "enum variant mismatch")]
fn test_none_some_mismatch() {
    let user = User {
        id: 8,
        name: "Henry".to_string(),
        email: None,
        age: Some(40),
        verified_at: None,
    };

    assert_struct!(
        user,
        User {
            id: 8,
            name: "Henry",
            email: Some("henry@example.com"),
            age: Some(40),
            verified_at: None,
        }
    );
}

#[test]
#[should_panic(expected = "value mismatch")]
fn test_some_value_mismatch() {
    let user = User {
        id: 9,
        name: "Iris".to_string(),
        email: Some("iris@example.com".to_string()),
        age: Some(25),
        verified_at: None,
    };

    assert_struct!(
        user,
        User {
            id: 9,
            name: "Iris",
            email: Some("wrong@example.com"),
            age: Some(25),
            verified_at: None,
        }
    );
}

error_message_test!(
    "option_errors/option_with_comparison.rs",
    option_with_comparison
);

// ============================================================================
// Advanced Option Patterns (merged from option_advanced.rs)
// ============================================================================

#[derive(Debug)]
struct UserAdvanced {
    name: String,
    age: Option<u32>,
    score: Option<f64>,
    #[allow(dead_code)] // Only used with regex feature
    email: Option<String>,
    #[allow(dead_code)] // Only used with regex feature
    user_id: Option<String>,
}

#[test]
fn test_option_comparison_greater() {
    let user = UserAdvanced {
        name: "Alice".to_string(),
        age: Some(35),
        score: Some(85.5),
        email: None,
        user_id: None,
    };

    assert_struct!(user, UserAdvanced {
        name: "Alice",
        age: Some(> 30),
        score: Some(> 80.0),
        ..
    });
}

#[test]
fn test_option_comparison_less() {
    let user = UserAdvanced {
        name: "Bob".to_string(),
        age: Some(25),
        score: Some(72.3),
        email: None,
        user_id: None,
    };

    assert_struct!(user, UserAdvanced {
        name: "Bob",
        age: Some(< 30),
        score: Some(< 80.0),
        ..
    });
}

#[test]
fn test_option_comparison_greater_equal() {
    let user = UserAdvanced {
        name: "Charlie".to_string(),
        age: Some(30),
        score: Some(80.0),
        email: None,
        user_id: None,
    };

    assert_struct!(user, UserAdvanced {
        name: "Charlie",
        age: Some(>= 30),
        score: Some(>= 80.0),
        ..
    });
}

#[test]
fn test_option_comparison_less_equal() {
    let user = UserAdvanced {
        name: "Diana".to_string(),
        age: Some(30),
        score: Some(80.0),
        email: None,
        user_id: None,
    };

    assert_struct!(user, UserAdvanced {
        name: "Diana",
        age: Some(<= 30),
        score: Some(<= 80.0),
        ..
    });
}

#[test]
#[cfg(feature = "regex")]
fn test_option_regex_pattern() {
    let user = UserAdvanced {
        name: "Eve".to_string(),
        age: None,
        score: None,
        email: Some("eve@example.com".to_string()),
        user_id: Some("usr_12345".to_string()),
    };

    assert_struct!(user, UserAdvanced {
        name: "Eve",
        email: Some(=~ r"@example\.com$"),
        user_id: Some(=~ r"^usr_\d+$"),
        ..
    });
}

#[test]
#[cfg(feature = "regex")]
fn test_mixed_option_patterns() {
    let user = UserAdvanced {
        name: "Frank".to_string(),
        age: Some(45),
        score: Some(92.5),
        email: Some("frank@company.org".to_string()),
        user_id: Some("usr_98765".to_string()),
    };

    assert_struct!(user, UserAdvanced {
        name: "Frank",
        age: Some(> 40),
        score: Some(>= 90.0),
        email: Some(=~ r"@company\.org$"),
        user_id: Some(=~ r"^\w+_\d{5}$"),
    });
}

#[test]
#[should_panic(expected = "comparison mismatch")]
fn test_option_comparison_failure() {
    let user = UserAdvanced {
        name: "Grace".to_string(),
        age: Some(25),
        score: None,
        email: None,
        user_id: None,
    };

    assert_struct!(user, UserAdvanced {
        name: "Grace",
        age: Some(> 30), // Should fail: 25 is not > 30
        ..
    });
}

#[test]
#[should_panic(expected = "enum variant mismatch")]
fn test_option_comparison_none_failure() {
    let user = UserAdvanced {
        name: "Henry".to_string(),
        age: None,
        score: None,
        email: None,
        user_id: None,
    };

    assert_struct!(user, UserAdvanced {
        name: "Henry",
        age: Some(> 30), // Should fail: got None instead of Some
        ..
    });
}

#[test]
#[cfg(feature = "regex")]
#[should_panic(expected = "regex pattern mismatch")]
fn test_option_regex_failure() {
    let user = UserAdvanced {
        name: "Iris".to_string(),
        age: None,
        score: None,
        email: Some("iris@wrong.net".to_string()),
        user_id: None,
    };

    assert_struct!(user, UserAdvanced {
        name: "Iris",
        email: Some(=~ r"@example\.com$"), // Should fail: wrong domain
        ..
    });
}

// ============================================================================
// Nested Structs in Options (merged from option_nested.rs)
// ============================================================================

// Note: Using existing Profile and Location structs defined earlier in the file

#[test]
fn test_some_nested_struct_without_to_string() {
    let profile = Profile {
        bio: Some("Software developer".to_string()),
        website: None,
        location: Some(Location {
            city: "Boston".to_string(),
            country: "USA".to_string(),
        }),
    };

    // This should work now without .to_string() in the nested struct!
    assert_struct!(
        profile,
        Profile {
            bio: Some("Software developer"),
            location: Some(Location {
                city: "Boston", // No .to_string() needed!
                country: "USA", // No .to_string() needed!
            }),
            ..
        }
    );
}

#[test]
fn test_some_nested_struct_partial_match() {
    let profile = Profile {
        bio: Some("Engineer".to_string()),
        website: None,
        location: Some(Location {
            city: "Paris".to_string(),
            country: "France".to_string(),
        }),
    };

    // Partial matching inside Some(...)
    assert_struct!(
        profile,
        Profile {
            bio: Some("Engineer"),
            location: Some(Location {
                city: "Paris",
                .. // Ignore country
            }),
            ..
        }
    );
}

#[test]
fn test_none_with_nested_struct_type() {
    let profile = Profile {
        bio: Some("Developer".to_string()),
        website: None,
        location: None,
    };

    assert_struct!(
        profile,
        Profile {
            bio: Some("Developer"),
            location: None,
            ..
        }
    );
}

// Test with more complex nesting
#[derive(Debug)]
struct UserNested {
    name: String,
    profile: Option<Profile>,
}

#[test]
fn test_deeply_nested_option_struct() {
    let user = UserNested {
        name: "Alice".to_string(),
        profile: Some(Profile {
            bio: Some("Rust developer".to_string()),
            website: None,
            location: Some(Location {
                city: "Seattle".to_string(),
                country: "USA".to_string(),
            }),
        }),
    };

    // Should handle deep nesting
    assert_struct!(
        user,
        UserNested {
            name: "Alice",
            profile: Some(Profile {
                bio: Some("Rust developer"),
                location: Some(Location {
                    city: "Seattle",
                    country: "USA",
                }),
                ..
            }),
        }
    );
}

#[test]
fn test_deeply_nested_with_partial_matching() {
    let user = UserNested {
        name: "Bob".to_string(),
        profile: Some(Profile {
            bio: Some("Engineer".to_string()),
            website: None,
            location: Some(Location {
                city: "London".to_string(),
                country: "UK".to_string(),
            }),
        }),
    };

    // Partial matching at multiple levels
    assert_struct!(
        user,
        UserNested {
            name: "Bob",
            profile: Some(Profile {
                bio: Some("Engineer"),
                location: Some(Location { city: "London", .. }),
                ..
            }),
        }
    );
}

#[test]
#[should_panic(expected = "value mismatch")]
fn test_nested_field_mismatch() {
    let profile = Profile {
        bio: Some("Developer".to_string()),
        website: None,
        location: Some(Location {
            city: "Boston".to_string(),
            country: "USA".to_string(),
        }),
    };

    assert_struct!(
        profile,
        Profile {
            bio: Some("Developer"),
            location: Some(Location {
                city: "Paris", // Wrong city
                country: "USA",
            }),
            ..
        }
    );
}

#[test]
#[should_panic(expected = "enum variant mismatch")]
fn test_expected_some_got_none_nested() {
    let profile = Profile {
        bio: Some("Developer".to_string()),
        website: None,
        location: None,
    };

    assert_struct!(
        profile,
        Profile {
            bio: Some("Developer"),
            location: Some(Location {
                city: "Boston",
                country: "USA",
            }),
            ..
        }
    );
}
