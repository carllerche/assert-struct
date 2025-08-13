use assert_struct::assert_struct;

#[derive(Debug)]
struct Profile {
    bio: String,
    verified: bool,
    followers: u32,
}

#[derive(Debug)]
struct Account {
    username: String,
    profile: Profile,
}

pub fn test_case() {
    let account = Account {
        username: "bob123".to_string(),
        profile: Profile {
            bio: "Hello".to_string(),  // Will fail: expected longer bio
            verified: false,  // Will fail: expected true
            followers: 5,  // Will fail: expected > 100
        },
    };

    assert_struct!(account, Account {
        username: "bob123",
        profile: Profile {
            bio: "A detailed bio that is much longer",
            verified: true,
            followers: > 100,
        },
    });
}