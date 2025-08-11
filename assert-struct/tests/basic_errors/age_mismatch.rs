use assert_struct::assert_struct;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

pub fn test_case() {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
    };

    assert_struct!(
        user,
        User {
            name: "Alice",
            age: 25,
        }
    );
}