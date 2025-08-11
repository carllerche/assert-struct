use assert_struct::assert_struct;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

#[test]
fn test_pattern_string_generation() {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
    };

    // This should succeed, but we're testing that the pattern string is generated
    assert_struct!(
        user,
        User {
            name: "Alice",
            age: 30,
        }
    );

    // Test with comparison
    assert_struct!(user, User {
        name: "Alice",
        age: >= 30,
    });
}
