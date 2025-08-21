use assert_struct::assert_struct;

#[derive(Debug)]
struct Expr;

#[derive(Debug)]
struct QuerySql {
    filter: Expr,
}

fn user_id_string() -> String {
    "123".to_string()
}

fn main() {
    let query_sql = QuerySql {
        filter: Expr,
    };

    // This should fail with a type error pointing to the specific Like pattern expression
    // The error should point to `user_id_string()` not the entire macro call
    assert_struct!(query_sql, QuerySql {
        filter: =~ user_id_string(),  // Type error: Expr doesn't implement Like<String>
        ..
    });
}