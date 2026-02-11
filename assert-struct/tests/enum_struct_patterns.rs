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
    Batch(Vec<Query>),
}

#[derive(Debug)]
struct Request {
    statement: Statement,
}

#[test]
fn test_enum_tuple_with_struct_basic() {
    let request = Request {
        statement: Statement::Query(Query {
            name: "get_items".to_string(),
            params: vec!["foo".to_string()],
            limit: Some(10),
        }),
    };

    assert_struct!(
        request,
        Request {
            statement: Statement::Query(Query {
                name: "get_items",
                ..
            })
        }
    );
}

#[test]
fn test_enum_tuple_with_struct_full_match() {
    let request = Request {
        statement: Statement::Query(Query {
            name: "select_users".to_string(),
            params: vec!["id".to_string(), "name".to_string()],
            limit: None,
        }),
    };

    assert_struct!(
        request,
        Request {
            statement: Statement::Query(Query {
                name: "select_users",
                params: ["id", "name"],
                limit: None,
            })
        }
    );
}

#[test]
fn test_enum_tuple_with_struct_patterns() {
    let request = Request {
        statement: Statement::Query(Query {
            name: "select_users".to_string(),
            params: vec!["age".to_string(), "email".to_string()],
            limit: Some(100),
        }),
    };

    assert_struct!(request, Request {
        statement: Statement::Query(Query {
            name: == "select_users",
            params: [== "age", == "email"],
            limit: Some(> 50),
        })
    });
}

#[test]
fn test_enum_struct_variant_with_nested_struct() {
    let request = Request {
        statement: Statement::Execute {
            id: 42,
            query: Query {
                name: "update_user".to_string(),
                params: vec!["age".to_string()],
                limit: Some(1),
            },
        },
    };

    assert_struct!(request, Request {
        statement: Statement::Execute {
            id: > 10,
            query: Query {
                name: "update_user",
                params: [== "age"],
                ..
            }
        }
    });
}

#[test]
fn test_enum_struct_variant_partial_match() {
    let request = Request {
        statement: Statement::Execute {
            id: 99,
            query: Query {
                name: "delete_item".to_string(),
                params: vec!["id".to_string()],
                limit: None,
            },
        },
    };

    // Only check the id field, ignore the query
    assert_struct!(request, Request {
        statement: Statement::Execute {
            id: < 100,
            ..
        }
    });
}

#[test]
#[cfg(feature = "regex")]
fn test_enum_struct_with_regex() {
    let request = Request {
        statement: Statement::Query(Query {
            name: "select_from_users".to_string(),
            params: vec!["*".to_string()],
            limit: None,
        }),
    };

    assert_struct!(request, Request {
        statement: Statement::Query(Query {
            name: =~ r"select.*users",
            ..
        })
    });
}

// Nested enums with structs
#[derive(Debug)]
enum InnerStatement {
    Simple(String),
    Complex(Query),
}

#[derive(Debug)]
struct NestedRequest {
    inner: InnerStatement,
}

#[test]
fn test_nested_enum_with_struct() {
    let request = NestedRequest {
        inner: InnerStatement::Complex(Query {
            name: "nested_query".to_string(),
            params: vec!["param1".to_string()],
            limit: Some(5),
        }),
    };

    assert_struct!(request, NestedRequest {
        inner: InnerStatement::Complex(Query {
            name: "nested_query",
            limit: Some(<= 10),
            ..
        })
    });
}

// Multiple levels of nesting
#[derive(Debug)]
struct Container {
    data: Option<Statement>,
}

#[test]
fn test_option_enum_with_struct() {
    let container = Container {
        data: Some(Statement::Query(Query {
            name: "get_all".to_string(),
            params: vec![],
            limit: None,
        })),
    };

    assert_struct!(
        container,
        Container {
            data: Some(Statement::Query(Query {
                name: "get_all",
                params: [],
                ..
            }))
        }
    );
}

// Error message tests
#[macro_use]
mod util;

error_message_test!(
    "enum_struct_patterns_errors/enum_struct_field_mismatch.rs",
    enum_struct_field_mismatch
);

error_message_test!(
    "enum_struct_patterns_errors/enum_struct_comparison_failure.rs",
    enum_struct_comparison_failure
);

error_message_test!(
    "enum_struct_patterns_errors/enum_struct_variant_nested_mismatch.rs",
    enum_struct_variant_nested_mismatch
);

error_message_test!(
    "enum_struct_patterns_errors/enum_variant_wrong_struct.rs",
    enum_variant_wrong_struct
);

error_message_test!(
    "enum_struct_patterns_errors/nested_option_enum_struct_failure.rs",
    nested_option_enum_struct_failure
);

error_message_test!(
    "enum_struct_patterns_errors/enum_struct_multiple_field_failures.rs",
    enum_struct_multiple_field_failures
);
