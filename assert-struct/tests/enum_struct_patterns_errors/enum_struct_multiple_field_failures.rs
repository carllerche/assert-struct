#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Query {
    name: String,
    params: Vec<String>,
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
            params: vec!["id".to_string()],
            limit: Some(25),
        }),
    };

    assert_struct!(request, Request {
        statement: Statement::Query(Query {
            name: "get_items",  // This will fail
            params: vec!["name", "age"],  // This will also fail (wrong length)
            limit: Some(> 50),  // This will also fail
        })
    });
}