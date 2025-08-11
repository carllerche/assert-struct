use assert_struct::assert_struct;

#[derive(Debug)]
struct TupleHolder {
    id: u32,
    data: (u32, String),
    active: bool,
}

pub fn test_case() {
    let holder = TupleHolder {
        id: 1,
        data: (50, "test".to_string()),
        active: true,
    };

    assert_struct!(
        holder,
        TupleHolder {
            id: 1,
            data: (60, "test"), // Wrong first element
            active: true,
        }
    );
}