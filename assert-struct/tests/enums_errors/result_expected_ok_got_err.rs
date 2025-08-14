use assert_struct::assert_struct;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

#[derive(Debug)]
struct UserData {
    login_result: Result<String, String>,
    parse_result: Result<i32, String>,
    complex_result: Result<User, String>,
}

pub fn test_case() {
    let data = UserData {
        login_result: Err("Failed".to_string()),
        parse_result: Ok(42),
        complex_result: Ok(User {
            name: "Test".to_string(),
            age: 20,
        }),
    };

    assert_struct!(
        data,
        UserData {
            login_result: Ok("user123"),
            parse_result: Ok(42),
            complex_result: Ok(User {
                name: "Test",
                age: 20,
            }),
        }
    );
}