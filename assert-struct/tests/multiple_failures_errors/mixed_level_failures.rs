use assert_struct::assert_struct;

#[derive(Debug)]
struct Address {
    street: String,
    city: String,
    zip: u32,
}

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
    email: String,
    address: Address,
}

pub fn test_case() {
    let user = User {
        name: "Alice".to_string(),
        age: 25,
        email: "alice@example.com".to_string(),
        address: Address {
            street: "123 Main St".to_string(),
            city: "Springfield".to_string(),
            zip: 12345,
        },
    };

    assert_struct!(user, User {
        name: "Alice",
        age: 30,  // Line 32 - root level failure (25 != 30)
        email: "alice@example.com",
        address: Address {
            street: "123 Main St",
            city: "Shelbyville",  // Line 36 - nested failure
            zip: 12345,
        },
    });
}