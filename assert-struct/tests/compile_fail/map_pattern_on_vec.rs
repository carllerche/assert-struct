use assert_struct::assert_struct;

#[derive(Debug)]
struct Data {
    items: Vec<i32>,
}

fn main() {
    let data = Data {
        items: vec![1, 2, 3],
    };

    // This should NOT compile - Vec has len() but get() returns Option<&T> with usize, not with &K
    assert_struct!(data, Data {
        items: #{ "key": 42 },
    });
}