use assert_struct::assert_struct;

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
