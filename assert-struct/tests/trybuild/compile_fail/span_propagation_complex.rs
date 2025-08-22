use assert_struct::assert_struct;

#[derive(Debug)]
struct ValueStream {
    data: Vec<i32>,
}

#[derive(Debug)]
enum Rows {
    Values(ValueStream),
}

#[derive(Debug)]
struct Response {
    rows: Rows,
}

#[tokio::main]
async fn main() {
    let resp = Response {
        rows: Rows::Values(ValueStream {
            data: vec![1, 2, 3],
        }),
    };

    // This should fail with error pointing to the specific operation
    // NOT to the entire assert_struct! call - replicating the original issue
    assert_struct!(resp.rows, Rows::Values(
        0.collect().await.unwrap(): [
            =~ 1
        ]
    ));
}