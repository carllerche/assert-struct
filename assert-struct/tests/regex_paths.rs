#![allow(dead_code)]
#[cfg(feature = "regex")]
use assert_struct::assert_struct;

#[cfg(feature = "regex")]
#[derive(Debug)]
struct User {
    name: String,
    email: String,
    profile: Profile,
}

#[cfg(feature = "regex")]
#[derive(Debug)]
struct Profile {
    bio: String,
    website: String,
}

#[cfg(feature = "regex")]
#[test]
fn test_regex_with_path_success() {
    let user = User {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
        profile: Profile {
            bio: "Software Developer".to_string(),
            website: "https://alice.dev".to_string(),
        },
    };

    assert_struct!(user, User {
        name: "Alice",
        email: =~ r"@example\.com$",
        profile: Profile {
            bio: =~ r"Developer",
            website: =~ r"^https://",
        },
    });
}

#[cfg(feature = "regex")]
#[test]
#[ignore = "error message format in flux"]
#[should_panic(expected = "assert_struct! failed")]
fn test_regex_failure_shows_path() {
    let user = User {
        name: "Bob".to_string(),
        email: "bob@wrong.org".to_string(),
        profile: Profile {
            bio: "Manager".to_string(),
            website: "http://bob.com".to_string(),
        },
    };

    assert_struct!(user, User {
        name: "Bob",
        email: =~ r"@example\.com$",  // Should fail and show path
        profile: Profile {
            bio: "Manager",
            website: =~ r"^http://",
        },
    });
}

#[cfg(feature = "regex")]
#[test]
#[ignore = "error message format in flux"]
#[should_panic(expected = "assert_struct! failed")]
fn test_nested_regex_failure_shows_path() {
    let user = User {
        name: "Charlie".to_string(),
        email: "charlie@example.com".to_string(),
        profile: Profile {
            bio: "Designer".to_string(),
            website: "ftp://charlie.dev".to_string(),
        },
    };

    assert_struct!(user, User {
        name: "Charlie",
        email: =~ r"@example\.com$",
        profile: Profile {
            bio: "Designer",
            website: =~ r"^https://",  // Should fail and show nested path
        },
    });
}
