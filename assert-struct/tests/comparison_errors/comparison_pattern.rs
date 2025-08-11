use assert_struct::assert_struct;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

pub fn test_case() {
    let user = User {
        name: "Alice".to_string(),
        age: 25,
    };

    assert_struct!(user, User {
        age: > 30,
        ..
    });
}