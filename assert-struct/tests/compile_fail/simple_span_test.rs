use assert_struct::assert_struct;

#[derive(Debug)]
struct TestStruct {
    value: i32,
}

fn main() {
    let data = TestStruct { value: 42 };
    
    // This should fail with a type error
    assert_struct!(data, TestStruct {
        value[0]: 42,
        ..
    });
}