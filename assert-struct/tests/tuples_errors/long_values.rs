use assert_struct::assert_struct;

#[derive(Debug)]
struct TupleHolder {
    id: u32,
    triple: (i32, String, bool),
}

pub fn test_case() {
    let holder = TupleHolder {
        id: 5,
        triple: (
            999,
            "this_is_a_very_long_string_that_should_be_truncated_in_the_error_output".to_string(),
            false
        ),
    };

    assert_struct!(
        holder,
        TupleHolder {
            id: 5,
            triple: (
                1000,
                "different_but_also_very_long_string_that_needs_truncation",
                true
            ),
        }
    );
}