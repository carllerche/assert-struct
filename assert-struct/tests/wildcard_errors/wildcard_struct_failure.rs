use assert_struct::assert_struct;

#[derive(Debug)]
struct Inner {
    value: i32,
    text: String,
}

#[derive(Debug)]
struct Outer {
    inner: Inner,
    count: u32,
}

pub fn test_case() {
    let data = Outer {
        inner: Inner {
            value: 10,
            text: "test".to_string(),
        },
        count: 5,
    };

    assert_struct!(data, _ {
        inner: _ {
            value: 20,  // This should fail
            ..
        },
        ..
    });
}
