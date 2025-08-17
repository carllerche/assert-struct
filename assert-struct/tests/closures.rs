use assert_struct::assert_struct;

#[derive(Debug)]
struct TestData {
    value: i32,
    name: String,
    items: Vec<i32>,
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
            value: |x: &i32| *x > 40,
            name: |n: &String| n.len() == 4,
            items: |items: &Vec<i32>| items.len() == 3,
        }
    );
}

#[test]
fn test_basic_closure_failure() {
    let data = TestData {
        value: 30,
        name: "test".to_string(),
        items: vec![1, 2],
    };

    let message = std::panic::catch_unwind(|| {
        assert_struct!(
            data,
            TestData {
                value: |x: &i32| *x > 40, // This should fail
                ..
            }
        );
    })
    .unwrap_err()
    .downcast::<String>()
    .unwrap();

    insta::assert_snapshot!(message);
}

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
            value: |x: &i32| *x >= 50 && *x <= 100,
            name: |n: &String| n.starts_with("h") && n.len() > 3,
            items: |items: &Vec<i32>| {
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
            value: move |x: &i32| *x > threshold, // Captures threshold by value
            ..
        }
    );
}
