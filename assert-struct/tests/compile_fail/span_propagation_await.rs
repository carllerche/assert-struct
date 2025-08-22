use assert_struct::assert_struct;

#[derive(Debug)]
struct TestStruct {
    value: i32,
}

#[tokio::main]
async fn main() {
    let data = TestStruct { value: 42 };
    
    // This should fail with error pointing to .await
    // NOT to the entire assert_struct! call
    assert_struct!(data, TestStruct {
        value.await: 42,
        ..
    });
}