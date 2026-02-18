use assert_struct::assert_struct;

#[derive(Debug)]
struct TestData {
    value: i32,
    name: String,
    items: Vec<i32>,
}

pub fn test_case() {
    let data = TestData {
        value: 30,
        name: "test".to_string(),
        items: vec![1, 2],
    };

    assert_struct!(
        data,
        TestData {
            value: |x| *x > 40, // This should fail
            ..
        }
    );
}
