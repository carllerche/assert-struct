#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Query {
    name: String,
}

#[derive(Debug)]
struct Update {
    table: String,
}

#[derive(Debug)]
enum Statement {
    Query(Query),
    Update(Update),
}

#[derive(Debug)]
struct Request {
    statement: Statement,
}

pub fn test_case() {
    let request = Request {
        statement: Statement::Update(Update {
            table: "users".to_string(),
        }),
    };

    // Expecting Query but got Update
    assert_struct!(request, Request {
        statement: Statement::Query(Query {
            name: "select_all",
        })
    });
}