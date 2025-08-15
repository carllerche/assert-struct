use assert_struct::assert_struct;

#[derive(Debug)]
struct Foo {
    #[allow(dead_code)]
    value: i32,
}

#[test]
#[should_panic(expected = "Some(10)")]
fn test_simple_int_value() {
    let value: Option<i32> = Some(10);
    assert_struct!(value, None);
}

#[test]
#[should_panic(expected = "Some(\"hello\")")]
fn test_simple_string_value() {
    let value: Option<&str> = Some("hello");
    assert_struct!(value, None);
}

#[test]
#[should_panic(expected = "Ok(42)")]
fn test_nested_result_simple() {
    let value: Result<i32, String> = Ok(42);
    assert_struct!(value, Err("error"));
}

#[test]
#[should_panic(expected = "Some(Ok(\"success\"))")]
fn test_nested_option_result() {
    let value: Option<Result<&str, i32>> = Some(Ok("success"));
    assert_struct!(value, None);
}

#[test]
#[should_panic(expected = "Some(..)")]
fn test_struct_value_abbreviated() {
    let value: Option<Foo> = Some(Foo { value: 42 });
    assert_struct!(value, None);
}

#[test]
#[should_panic(expected = "Ok(..)")]
fn test_result_with_struct_abbreviated() {
    let value: Result<Foo, String> = Ok(Foo { value: 42 });
    assert_struct!(value, Err("error"));
}
