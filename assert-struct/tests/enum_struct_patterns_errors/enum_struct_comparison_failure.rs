#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Query {
    name: String,
    limit: Option<usize>,
}

#[derive(Debug)]
enum Statement {
    Query(Query),
}

#[derive(Debug)]
struct Request {
    statement: Statement,
}

pub fn test_case() {
    let request = Request {
        statement: Statement::Query(Query {
            name: "select_users".to_string(),
            limit: Some(25),
        }),
    };

    assert_struct!(request, Request {
        statement: Statement::Query(Query {
            name: "select_users",
            limit: Some(> 50),  // This will fail: 25 is not > 50
        })
    });
}