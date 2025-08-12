use assert_struct::assert_struct;

#[macro_use]
mod util;

// Test Result enum
#[derive(Debug)]
struct UserData {
    login_result: Result<String, String>,
    parse_result: Result<i32, String>,
    complex_result: Result<User, ErrorInfo>,
}

#[derive(Debug, PartialEq)]
struct User {
    name: String,
    age: u32,
}

#[derive(Debug, PartialEq)]
struct ErrorInfo {
    code: i32,
    message: String,
}

#[test]
fn test_result_ok_simple() {
    let data = UserData {
        login_result: Ok("user123".to_string()),
        parse_result: Ok(42),
        complex_result: Ok(User {
            name: "Alice".to_string(),
            age: 30,
        }),
    };

    assert_struct!(
        data,
        UserData {
            login_result: Ok("user123"),
            parse_result: Ok(42),
            complex_result: Ok(User {
                name: "Alice",
                age: 30,
            }),
        }
    );
}

#[test]
fn test_result_err_simple() {
    let data = UserData {
        login_result: Err("Invalid credentials".to_string()),
        parse_result: Err("Not a number".to_string()),
        complex_result: Err(ErrorInfo {
            code: 404,
            message: "Not found".to_string(),
        }),
    };

    assert_struct!(
        data,
        UserData {
            login_result: Err("Invalid credentials"),
            parse_result: Err("Not a number"),
            complex_result: Err(ErrorInfo {
                code: 404,
                message: "Not found",
            }),
        }
    );
}

#[test]
fn test_result_with_comparisons() {
    let data = UserData {
        login_result: Ok("user123".to_string()),
        parse_result: Ok(100),
        complex_result: Ok(User {
            name: "Bob".to_string(),
            age: 25,
        }),
    };

    assert_struct!(data, UserData {
        login_result: Ok("user123"),
        parse_result: Ok(> 50),  // Comparison inside Ok
        complex_result: Ok(User {
            name: "Bob",
            age: >= 18,  // Adult check
        }),
    });
}

// Custom enums
#[derive(Debug, PartialEq)]
enum Status {
    Active,
    Inactive,
    Pending { since: String },
    Error { code: i32, message: String },
}

#[derive(Debug)]
struct Account {
    id: u32,
    status: Status,
}

#[test]
fn test_custom_enum_unit_variants() {
    let account1 = Account {
        id: 1,
        status: Status::Active,
    };

    assert_struct!(
        account1,
        Account {
            id: 1,
            status: Status::Active,
        }
    );

    let account2 = Account {
        id: 2,
        status: Status::Inactive,
    };

    assert_struct!(
        account2,
        Account {
            id: 2,
            status: Status::Inactive,
        }
    );
}

#[test]
fn test_custom_enum_struct_variant() {
    let account = Account {
        id: 3,
        status: Status::Pending {
            since: "2024-01-01".to_string(),
        },
    };

    assert_struct!(
        account,
        Account {
            id: 3,
            status: Status::Pending {
                since: "2024-01-01",
            },
        }
    );
}

#[test]
fn test_custom_enum_struct_variant_partial() {
    let account = Account {
        id: 4,
        status: Status::Error {
            code: 500,
            message: "Internal server error".to_string(),
        },
    };

    // Partial match - only check the code
    assert_struct!(
        account,
        Account {
            id: 4,
            status: Status::Error { code: 500, .. },
        }
    );
}

// Tuple enums with multiple fields
#[derive(Debug, PartialEq)]
enum Message {
    Text(String),
    Data(String, Vec<u8>),
    Complex(u32, String, bool),
}

#[derive(Debug)]
struct MessageQueue {
    current: Message,
    priority: u8,
}

#[test]
fn test_tuple_enum_single_field() {
    let msg = MessageQueue {
        current: Message::Text("Hello".to_string()),
        priority: 1,
    };

    assert_struct!(
        msg,
        MessageQueue {
            current: Message::Text("Hello"),
            priority: 1,
        }
    );
}

#[test]
fn test_tuple_enum_multiple_fields() {
    let msg = MessageQueue {
        current: Message::Data("metadata".to_string(), vec![1, 2, 3]),
        priority: 2,
    };

    assert_struct!(
        msg,
        MessageQueue {
            current: Message::Data("metadata", vec![1, 2, 3]),
            priority: 2,
        }
    );
}

#[test]
fn test_tuple_enum_three_fields() {
    let msg = MessageQueue {
        current: Message::Complex(42, "test".to_string(), true),
        priority: 3,
    };

    assert_struct!(
        msg,
        MessageQueue {
            current: Message::Complex(42, "test", true),
            priority: 3,
        }
    );
}

// Mixed enum with all variant types
#[derive(Debug, PartialEq)]
enum Response {
    Success,
    Redirect(String),
    Data {
        payload: Vec<u8>,
        content_type: String,
    },
    MultiPart(String, Vec<u8>, bool),
}

#[derive(Debug)]
struct ApiResponse {
    response: Response,
    timestamp: u64,
}

#[test]
fn test_mixed_enum_unit() {
    let resp = ApiResponse {
        response: Response::Success,
        timestamp: 1234567890,
    };

    assert_struct!(
        resp,
        ApiResponse {
            response: Response::Success,
            timestamp: 1234567890,
        }
    );
}

#[test]
fn test_mixed_enum_single_tuple() {
    let resp = ApiResponse {
        response: Response::Redirect("/home".to_string()),
        timestamp: 1234567891,
    };

    assert_struct!(
        resp,
        ApiResponse {
            response: Response::Redirect("/home"),
            timestamp: 1234567891,
        }
    );
}

#[test]
fn test_mixed_enum_struct_fields() {
    let resp = ApiResponse {
        response: Response::Data {
            payload: vec![65, 66, 67],
            content_type: "text/plain".to_string(),
        },
        timestamp: 1234567892,
    };

    assert_struct!(
        resp,
        ApiResponse {
            response: Response::Data {
                payload: vec![65, 66, 67],
                content_type: "text/plain",
            },
            timestamp: 1234567892,
        }
    );
}

#[test]
fn test_mixed_enum_multi_tuple() {
    let resp = ApiResponse {
        response: Response::MultiPart("boundary".to_string(), vec![1, 2], false),
        timestamp: 1234567893,
    };

    assert_struct!(
        resp,
        ApiResponse {
            response: Response::MultiPart("boundary", vec![1, 2], false),
            timestamp: 1234567893,
        }
    );
}

// Nested enums
#[derive(Debug, PartialEq)]
enum InnerEnum {
    A(String),
    B { value: i32 },
}

#[derive(Debug, PartialEq)]
enum OuterEnum {
    Wrapper(InnerEnum),
}

#[derive(Debug)]
struct NestedEnumStruct {
    outer: OuterEnum,
}

#[test]
fn test_nested_enum_tuple_variant() {
    let nested = NestedEnumStruct {
        outer: OuterEnum::Wrapper(InnerEnum::A("nested".to_string())),
    };

    // Note: For nested enum tuples, we currently need .to_string() for string literals
    assert_struct!(
        nested,
        NestedEnumStruct {
            outer: OuterEnum::Wrapper(InnerEnum::A("nested".to_string())),
        }
    );
}

#[test]
fn test_nested_enum_struct_variant() {
    let nested = NestedEnumStruct {
        outer: OuterEnum::Wrapper(InnerEnum::B { value: 42 }),
    };

    assert_struct!(
        nested,
        NestedEnumStruct {
            outer: OuterEnum::Wrapper(InnerEnum::B { value: 42 }),
        }
    );
}

// Failure tests
#[test]
#[should_panic(expected = "enum variant mismatch")]
fn test_result_expected_ok_got_err() {
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

#[test]
#[should_panic(expected = "enum variant mismatch")]
fn test_enum_variant_mismatch() {
    let account = Account {
        id: 5,
        status: Status::Inactive,
    };

    assert_struct!(
        account,
        Account {
            id: 5,
            status: Status::Active,
        }
    );
}

#[test]
#[should_panic(expected = "value mismatch")]
fn test_tuple_enum_field_mismatch() {
    let msg = MessageQueue {
        current: Message::Complex(42, "wrong".to_string(), true),
        priority: 3,
    };

    assert_struct!(
        msg,
        MessageQueue {
            current: Message::Complex(42, "test", true),
            priority: 3,
        }
    );
}

error_message_test!("enums_errors/enum_variant.rs", enum_variant);
