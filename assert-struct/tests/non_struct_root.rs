#![allow(dead_code)]
use assert_struct::assert_struct;

#[test]
fn test_range_at_root() {
    let value = 15;
    assert_struct!(value, 10..20);
}

#[test]
fn test_comparison_at_root() {
    let value = 25;
    assert_struct!(value, > 20);
    assert_struct!(value, <= 30);
    assert_struct!(value, == 25);
}

#[test]
fn test_option_at_root() {
    let value = Some(42);
    assert_struct!(value, Some(42));

    let none_value: Option<i32> = None;
    assert_struct!(none_value, None);
}

#[test]
fn test_option_with_comparison_at_root() {
    let value = Some(50);
    assert_struct!(value, Some(> 40));
}

#[test]
fn test_option_slice_at_root() {
    let data = Some(vec![1, 2, 3]);
    assert_struct!(data, Some([1, 2, 3]));

    let data = Some(vec![1, 2, 3, 4, 5]);
    assert_struct!(data, Some([1, 2, ..]));
}

#[test]
fn test_slice_at_root() {
    let items = vec![1, 2, 3];
    assert_struct!(items, [1, 2, 3]);

    let items = vec![10, 20, 30, 40];
    assert_struct!(items, [10, 20, ..]);
    assert_struct!(items, [.., 40]);
}

#[test]
fn test_slice_with_comparisons_at_root() {
    let items = vec![5, 15, 25];
    assert_struct!(items, [> 0, < 20, == 25]);
}

#[test]
fn test_tuple_at_root() {
    let pair = (10, 20);
    assert_struct!(pair, (10, 20));

    let triple = (1, "hello", true);
    assert_struct!(triple, (1, "hello", true));
}

#[test]
fn test_tuple_with_comparisons_at_root() {
    let pair = (15, 25);
    assert_struct!(pair, (> 10, < 30));
}

#[test]
fn test_result_at_root() {
    let result: Result<i32, String> = Ok(42);
    assert_struct!(result, Ok(42));

    let error: Result<i32, String> = Err("error".to_string());
    assert_struct!(error, Err("error"));
}

#[test]
fn test_result_with_slice_at_root() {
    let result: Result<Vec<i32>, String> = Ok(vec![1, 2, 3]);
    assert_struct!(result, Ok([1, 2, 3]));
}

#[test]
#[allow(clippy::double_parens)]
fn test_nested_pattern_at_root() {
    // Simple nested patterns work
    let simple = Some((10, 20));
    assert_struct!(simple, Some((10, 20)));

    // Complex nested patterns with slices need workaround for now
    // let complex = Some((10, vec![1, 2, 3], "test"));
    // assert_struct!(complex, Some((10, [1, 2, 3], "test")));
}

#[test]
#[cfg(feature = "regex")]
fn test_regex_at_root() {
    let text = "hello123";
    assert_struct!(text, =~ r"^hello\d+$");
}

#[test]
fn test_simple_value_at_root() {
    let value = 42;
    assert_struct!(value, 42);

    let text = "hello";
    assert_struct!(text, "hello");

    let flag = true;
    assert_struct!(flag, true);
}

// Test that we can still do struct patterns
#[derive(Debug)]
struct Point {
    x: i32,
    y: i32,
}

#[test]
fn test_struct_still_works() {
    let point = Point { x: 10, y: 20 };
    assert_struct!(point, Point { x: 10, y: 20 });
    assert_struct!(point, Point { x: > 5, .. });
}

// Test for issue #93: asserting on array-indexing should not require references
#[test]
fn test_index_into_vec_enum_struct_variant() {
    #[derive(Debug)]
    enum SystemEvent {
        Startup { version: String, code: u32 },
    }

    let events = vec![SystemEvent::Startup {
        version: "1.0.0".to_string(),
        code: 42,
    }];

    // Issue #93: string literal comparison should not require `&"1.0.0"`
    assert_struct!(events[0], SystemEvent::Startup { version: "1.0.0", .. });

    // Comparison operators on numeric fields should also work
    assert_struct!(events[0], SystemEvent::Startup { code: > 10, .. });
    assert_struct!(events[0], SystemEvent::Startup { code: == 42, .. });
}
