#![allow(dead_code)]
use assert_struct::assert_struct;

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

pub fn test_case() {
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