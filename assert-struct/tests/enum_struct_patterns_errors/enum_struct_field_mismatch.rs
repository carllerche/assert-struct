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
    Execute { id: u32, query: Query },
}

#[derive(Debug)]
struct Request {
    statement: Statement,
}

pub fn test_case() {
    let request = Request {
        statement: Statement::Query(Query {
            name: "select_users".to_string(),
            params: vec!["id".to_string(), "name".to_string()],
            limit: Some(50),
        }),
    };

    assert_struct!(request, Request {
        statement: Statement::Query(Query {
            name: "get_items",  // This will fail
            params: vec!["id", "name"],
            limit: Some(50),
        })
    });
}