use assert_struct::assert_struct;

#[derive(Debug)]
struct Container {
    items: Vec<i32>,
    names: Vec<String>,
    data: Vec<i32>,
}

pub fn test_case() {
    let container = Container {
        items: vec![1, 2, 3],
        names: vec!["a".to_string(), "b".to_string()],
        data: vec![10, 20],
    };

    assert_struct!(
        container,
        Container {
            items: [1, 2, 4],  // Last element wrong
            names: ["a", "c"], // Second element wrong
            data: [10, 20],
        }
    );
}