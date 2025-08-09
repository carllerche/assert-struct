use assert_struct::assert_struct;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

fn main() {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
    };

    // This should NOT compile - field 'email' does not exist
    assert_struct!(user, User {
        name: "Alice",
        age: 30,
        email: "alice@example.com",
    });
}