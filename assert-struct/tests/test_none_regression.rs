use assert_struct::assert_struct;

#[test]
#[should_panic(expected = "assert_struct! failed")]
fn test_none_error_message() {
    let value: Option<i32> = Some(42);
    assert_struct!(value, None);
}

#[test]
fn test_none_success() {
    let value: Option<i32> = None;
    assert_struct!(value, None);
}
