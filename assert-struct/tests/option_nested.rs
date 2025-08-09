use assert_struct::assert_struct;

// Test nested structs inside Option fields - string literals should work without .to_string()

#[derive(Debug, PartialEq)]
struct Location {
    city: String,
    country: String,
}

#[derive(Debug)]
struct Profile {
    bio: Option<String>,
    location: Option<Location>,
}

#[test]
fn test_some_nested_struct_without_to_string() {
    let profile = Profile {
        bio: Some("Software developer".to_string()),
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
        }
    );
}

#[test]
fn test_some_nested_struct_partial_match() {
    let profile = Profile {
        bio: Some("Engineer".to_string()),
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
        }
    );
}

#[test]
fn test_none_with_nested_struct_type() {
    let profile = Profile {
        bio: Some("Developer".to_string()),
        location: None,
    };

    assert_struct!(
        profile,
        Profile {
            bio: Some("Developer"),
            location: None,
        }
    );
}

// Test with more complex nesting
#[derive(Debug)]
struct User {
    name: String,
    profile: Option<Profile>,
}

#[test]
fn test_deeply_nested_option_struct() {
    let user = User {
        name: "Alice".to_string(),
        profile: Some(Profile {
            bio: Some("Rust developer".to_string()),
            location: Some(Location {
                city: "Seattle".to_string(),
                country: "USA".to_string(),
            }),
        }),
    };

    // Should handle deep nesting
    assert_struct!(
        user,
        User {
            name: "Alice",
            profile: Some(Profile {
                bio: Some("Rust developer"),
                location: Some(Location {
                    city: "Seattle",
                    country: "USA",
                }),
            }),
        }
    );
}

#[test]
fn test_deeply_nested_with_partial_matching() {
    let user = User {
        name: "Bob".to_string(),
        profile: Some(Profile {
            bio: Some("Engineer".to_string()),
            location: Some(Location {
                city: "London".to_string(),
                country: "UK".to_string(),
            }),
        }),
    };

    // Partial matching at multiple levels
    assert_struct!(
        user,
        User {
            name: "Bob",
            profile: Some(Profile {
                bio: Some("Engineer"),
                location: Some(Location { city: "London", .. }),
            }),
        }
    );
}

// Failure tests

#[test]
#[should_panic(expected = "assertion `left == right` failed")]
fn test_nested_field_mismatch() {
    let profile = Profile {
        bio: Some("Developer".to_string()),
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
        }
    );
}

#[test]
#[should_panic(expected = "Expected Some(...), got None")]
fn test_expected_some_got_none_nested() {
    let profile = Profile {
        bio: Some("Developer".to_string()),
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
        }
    );
}
