use assert_struct::assert_struct;

#[derive(Debug)]
struct QuerySql {
    stmt: Statement,
    ret: Option<String>,
}

#[derive(Debug)]
enum Statement {
    Query(QueryStmt),
}

#[derive(Debug)]
struct QueryStmt {
    body: ExprSet,
}

#[derive(Debug)]
enum ExprSet {
    Select(SelectStmt),
}

#[derive(Debug)]
struct SelectStmt {
    source: Source,
    filter: Expr,
}

#[derive(Debug)]
enum Source {
    Table(Vec<TableRef>),
}

#[derive(Debug)]
struct TableRef {
    table: String,
}

#[derive(Debug)]
enum Expr {
    BinaryOp(BinaryOpExpr),
}

#[derive(Debug)]
struct BinaryOpExpr {
    lhs: Box<String>,
    op: BinaryOp,
    rhs: Box<String>,
}

#[derive(Debug)]
enum BinaryOp {
    Eq,
}

fn main() {
    let query_sql = QuerySql {
        stmt: Statement::Query(QueryStmt {
            body: ExprSet::Select(SelectStmt {
                source: Source::Table(vec![TableRef { table: "users".to_string() }]),
                filter: Expr::BinaryOp(BinaryOpExpr {
                    lhs: Box::new("col".to_string()),
                    op: BinaryOp::Eq,
                    rhs: Box::new("val".to_string()),
                }),
            }),
        }),
        ret: Some("result".to_string()),
    };

    assert_struct!(query_sql, {
        stmt: Statement::Query({
            body: ExprSet::Select({
                source: Source::Table([
                    { table: "users" },
                ]),
                filter: Expr::BinaryOp({
                    *lhs: "col",
                    op: BinaryOp::Eq,
                    *rhs: == 123,  // Type error: can't compare String with integer
                }),
            }),
        }),
        ret: Some(_),
    });
}
