use assert_struct::assert_struct;

pub fn test_case() {
    let value = Some(10);

    assert_struct!(value, None);
}