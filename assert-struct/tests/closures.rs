use assert_struct::assert_struct;

#[macro_use]
mod util;

#[derive(Debug)]
struct TestData {
    value: i32,
    name: String,
    items: Vec<i32>,
}

#[derive(Debug)]
struct Container {
    data: TestData,
}

#[test]
fn test_basic_closure_success() {
    let data = TestData {
        value: 42,
        name: "test".to_string(),
        items: vec![1, 2, 3],
    };

    assert_struct!(
        data,
        TestData {
            value: |x| *x > 40,
            name: |n| n.len() == 4,
            items: |items| items.len() == 3,
        }
    );
}

error_message_test!(
    "closures_errors/basic_closure_failure.rs",
    basic_closure_failure
);

#[test]
fn test_closure_with_complex_logic() {
    let data = TestData {
        value: 50,
        name: "hello".to_string(),
        items: vec![10, 20, 30],
    };

    assert_struct!(
        data,
        TestData {
            value: |x| *x >= 50 && *x <= 100,
            name: |n| n.starts_with("h") && n.len() > 3,
            items: |items| {
                items.len() == 3 && items.iter().all(|&x| x > 0) && items.iter().sum::<i32>() == 60
            },
        }
    );
}

#[test]
fn test_move_closure() {
    let threshold = 25;
    let data = TestData {
        value: 30,
        name: "test".to_string(),
        items: vec![1, 2, 3],
    };

    assert_struct!(
        data,
        TestData {
            value: move |x| *x > threshold, // Captures threshold by value
            ..
        }
    );
}

#[test]
fn test_closure_nested_struct() {
    let container = Container {
        data: TestData {
            value: 42,
            name: "test".to_string(),
            items: vec![3, 4, 5],
        },
    };

    assert_struct!(
        container,
        Container {
            data: |data| data.value == 42
        }
    )
}
