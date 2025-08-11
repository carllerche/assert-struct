use assert_struct::assert_struct;

#[derive(Debug)]
struct Profile {
    age: u32,
    verified: bool,
}

#[derive(Debug)]
struct User {
    name: String,
    profile: Profile,
}

pub fn test_case() {
    let user = User {
        name: "Alice".to_string(),
        profile: Profile { age: 17, verified: true },
    };

    assert_struct!(user, User {
        name: "Alice",
        profile: Profile {
            age: >= 18,
            verified: true,
            ..
        },
        ..
    });
}