#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Query {
    name: String,
    limit: usize,
}

#[derive(Debug)]
enum Statement {
    Query(Query),
}

#[derive(Debug)]
struct Container {
    data: Option<Statement>,
}

pub fn test_case() {
    let container = Container {
        data: Some(Statement::Query(Query {
            name: "get_all".to_string(),
            limit: 10,
        })),
    };

    assert_struct!(container, Container {
        data: Some(Statement::Query(Query {
            name: "get_all",
            limit: > 100,  // This will fail: 10 is not > 100
        }))
    });
}