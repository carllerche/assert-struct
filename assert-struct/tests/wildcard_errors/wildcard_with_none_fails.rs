use assert_struct::assert_struct;

#[derive(Debug)]
struct TestStruct {
    name: String,
    value: i32,
    data: Option<String>,
    items: Vec<i32>,
    tuple: (i32, String, bool),
}

pub fn test_case() {
    let test = TestStruct {
        name: "test".to_string(),
        value: 42,
        data: None,
        items: vec![],
        tuple: (0, "".to_string(), false),
    };

    // This should fail - Some(_) expects Some, but got None
    assert_struct!(
        test,
        TestStruct {
            name: "test",
            value: 42,
            data: Some(_),
            items: [],
            tuple: (0, "", false),
        }
    );
}