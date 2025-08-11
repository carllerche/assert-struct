use assert_struct::assert_struct;

#[derive(Debug)]
struct Data {
    value: Option<u32>,
}

pub fn test_case() {
    let data = Data { value: Some(25) };

    assert_struct!(data, Data {
        value: Some(> 30),
    });
}