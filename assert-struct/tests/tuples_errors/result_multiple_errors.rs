#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Response {
    id: u32,
    data: Result<(i32, String), String>,
}

pub fn test_case() {
    let response = Response {
        id: 5,
        data: Ok((42, "success".to_string())),
    };

    assert_struct!(
        response,
        Response {
            id: 5,
            data: Ok((100, "different".to_string())),  // Both fields in Ok variant wrong
        }
    );
}