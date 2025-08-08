use assert_struct::assert_struct;

// Tests for advanced Option patterns - comparisons and regex inside Some()

#[derive(Debug)]
struct User {
    name: String,
    age: Option<u32>,
    score: Option<f64>,
    email: Option<String>,
    user_id: Option<String>,
}

#[test]
fn test_option_comparison_greater() {
    let user = User {
        name: "Alice".to_string(),
        age: Some(35),
        score: Some(85.5),
        email: None,
        user_id: None,
    };

    assert_struct!(user, User {
        name: "Alice",
        age: Some(> 30),
        score: Some(> 80.0),
        ..
    });
}

#[test]
fn test_option_comparison_less() {
    let user = User {
        name: "Bob".to_string(),
        age: Some(25),
        score: Some(72.3),
        email: None,
        user_id: None,
    };

    assert_struct!(user, User {
        name: "Bob",
        age: Some(< 30),
        score: Some(< 80.0),
        ..
    });
}

#[test]
fn test_option_comparison_greater_equal() {
    let user = User {
        name: "Charlie".to_string(),
        age: Some(30),
        score: Some(80.0),
        email: None,
        user_id: None,
    };

    assert_struct!(user, User {
        name: "Charlie",
        age: Some(>= 30),
        score: Some(>= 80.0),
        ..
    });
}

#[test]
fn test_option_comparison_less_equal() {
    let user = User {
        name: "Diana".to_string(),
        age: Some(30),
        score: Some(80.0),
        email: None,
        user_id: None,
    };

    assert_struct!(user, User {
        name: "Diana",
        age: Some(<= 30),
        score: Some(<= 80.0),
        ..
    });
}

#[test]
fn test_option_regex_pattern() {
    let user = User {
        name: "Eve".to_string(),
        age: None,
        score: None,
        email: Some("eve@example.com".to_string()),
        user_id: Some("usr_12345".to_string()),
    };

    assert_struct!(user, User {
        name: "Eve",
        email: Some(=~ r"@example\.com$"),
        user_id: Some(=~ r"^usr_\d+$"),
        ..
    });
}

#[test]
fn test_mixed_option_patterns() {
    let user = User {
        name: "Frank".to_string(),
        age: Some(45),
        score: Some(92.5),
        email: Some("frank@company.org".to_string()),
        user_id: Some("usr_98765".to_string()),
    };

    assert_struct!(user, User {
        name: "Frank",
        age: Some(> 40),
        score: Some(>= 90.0),
        email: Some(=~ r"@company\.org$"),
        user_id: Some(=~ r"^\w+_\d{5}$"),
    });
}

// Failure tests

#[test]
#[should_panic(expected = "failed comparison")]
fn test_option_comparison_failure() {
    let user = User {
        name: "Grace".to_string(),
        age: Some(25),
        score: None,
        email: None,
        user_id: None,
    };

    assert_struct!(user, User {
        name: "Grace",
        age: Some(> 30), // Should fail: 25 is not > 30
        ..
    });
}

#[test]
#[should_panic(expected = "expected Some(...), got None")]
fn test_option_comparison_none_failure() {
    let user = User {
        name: "Henry".to_string(),
        age: None,
        score: None,
        email: None,
        user_id: None,
    };

    assert_struct!(user, User {
        name: "Henry",
        age: Some(> 30), // Should fail: got None instead of Some
        ..
    });
}

#[test]
#[should_panic(expected = "does not match regex pattern")]
fn test_option_regex_failure() {
    let user = User {
        name: "Iris".to_string(),
        age: None,
        score: None,
        email: Some("iris@wrong.net".to_string()),
        user_id: None,
    };

    assert_struct!(user, User {
        name: "Iris",
        email: Some(=~ r"@example\.com$"), // Should fail: wrong domain
        ..
    });
}
