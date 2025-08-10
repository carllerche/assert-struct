use assert_struct::assert_struct;

#[derive(Debug)]
struct Data {
    values: Vec<i32>,
    scores: Vec<f64>,
}

#[test]
fn test_slice_element_comparison_success() {
    let data = Data {
        values: vec![10, 20, 30],
        scores: vec![85.5, 92.0, 78.3],
    };

    assert_struct!(data, Data {
        values: [> 5, < 25, >= 30],
        scores: [> 80.0, >= 90.0, < 80.0],
    });
}

#[test]
#[should_panic(expected = "data.values.[1]")]
fn test_slice_element_comparison_failure() {
    let data = Data {
        values: vec![10, 5, 30],
        scores: vec![85.5, 92.0, 78.3],
    };

    assert_struct!(data, Data {
        values: [> 5, > 10, >= 30],  // Second element (5) fails > 10
        scores: [> 80.0, >= 90.0, < 80.0],
    });
}
