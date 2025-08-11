use assert_struct::assert_struct;

#[derive(Debug)]
struct Data {
    values: Vec<i32>,
}

pub fn test_case() {
    let data = Data {
        values: vec![1, 2, 3],
    };

    assert_struct!(data, Data { 
        values: [1, 5, 3] 
    });
}