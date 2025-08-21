use assert_struct::assert_struct;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

pub fn test_case() {
    let user = User {
        name: "Alice".to_string(),
        age: 105, // This should fail the <= 99 constraint
    };

    assert_struct!(user, User { 
        age: >= 10,   // This should pass: 105 >= 10 ✓
        age: <= 99,   // This should fail: 105 <= 99 ✗
        .. 
    });
}