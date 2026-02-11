#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Query {
    name: String,
    params: Vec<String>,
}

#[derive(Debug)]
enum Statement {
    Execute { id: u32, query: Query },
    Query(Query),
}

#[derive(Debug)]
struct Request {
    statement: Statement,
}

pub fn test_case() {
    let request = Request {
        statement: Statement::Execute {
            id: 42,
            query: Query {
                name: "update_user".to_string(),
                params: vec!["age".to_string()],
            },
        },
    };

    assert_struct!(request, Request {
        statement: Statement::Execute {
            id: > 10,
            query: Query {
                name: "delete_user",  // This will fail
                params: ["age"],
            }
        }
    });
}
