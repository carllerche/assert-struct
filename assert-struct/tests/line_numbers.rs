use assert_struct::assert_struct;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
    email: String,
}

#[test]
#[should_panic(expected = "(line 19)")]
fn test_line_number_in_error() {
    let user = User {
        name: "Alice".to_string(),
        age: 25,
        email: "alice@example.com".to_string(),
    };

    assert_struct!(user, User {  // Line 19 - macro invocation line
        name: "Alice",
        age: > 30,  // Failure happens here
        email: "alice@example.com",
    });
}

#[test]
#[should_panic(expected = "(line 35)")]
fn test_nested_line_number() {
    let user = User {
        name: "Bob".to_string(),
        age: 40,
        email: "bob@wrong.com".to_string(),
    };

    assert_struct!(
        user,
        User {
            // Line 35 - macro invocation line
            name: "Bob",
            age: 40,
            email: "bob@example.com", // Failure happens here
        }
    );
}

#[cfg(feature = "regex")]
#[test]
#[should_panic(expected = "(line 56)")]
fn test_regex_line_number() {
    let user = User {
        name: "Charlie".to_string(),
        age: 35,
        email: "charlie@wrong.org".to_string(),
    };

    assert_struct!(user, User {  // Line 52 - macro invocation line
        name: "Charlie",
        age: 35,
        email: =~ r"@example\.com$",  // Failure happens here
    });
}
