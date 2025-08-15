use assert_struct::assert_struct;

#[derive(Debug)]
struct TupleHolder {
    id: u32,
    quad: (u8, u8, u8, u8),
}

pub fn test_case() {
    let holder = TupleHolder {
        id: 3,
        quad: (10, 20, 30, 40),
    };

    assert_struct!(
        holder,
        TupleHolder {
            id: 3,
            quad: (1, 2, 3, 4),  // All four fields wrong
        }
    );
}