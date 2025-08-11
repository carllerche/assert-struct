use assert_struct::assert_struct;

#[macro_use]
mod util;

#[derive(Debug)]
struct User {
    name: String,
    age: u32,
}

#[derive(Debug)]
struct DataHolder {
    values: Vec<u32>,
    name: String,
}

#[derive(Debug)]
struct TupleHolder {
    id: u32,
    data: (u32, String),
    active: bool,
}

#[test]
fn assert_eq_one_level() {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
    };

    assert_struct!(
        user,
        User {
            name: "Alice",
            age: 30,
        }
    );
}

#[test]
fn partial_match_with_rest() {
    let user = User {
        name: "Bob".to_string(),
        age: 25,
    };

    assert_struct!(user, User { name: "Bob", .. });
}

// Error message tests using snapshot testing
error_message_test!("basic_errors/field_mismatch.rs", field_mismatch);

error_message_test!("basic_errors/age_mismatch.rs", age_mismatch);

#[test]
fn test_vec_with_slice_syntax() {
    let data = DataHolder {
        values: vec![1, 2, 3],
        name: "test".to_string(),
    };

    assert_struct!(
        data,
        DataHolder {
            values: [1, 2, 3],
            name: "test",
        }
    );
}

#[test]
fn test_vec_partial_match() {
    let data = DataHolder {
        values: vec![1, 2, 3, 4, 5],
        name: "partial".to_string(),
    };

    assert_struct!(
        data,
        DataHolder {
            values: [1, 2, 3, 4, 5],
            ..
        }
    );
}

error_message_test!("basic_errors/vec_mismatch.rs", vec_mismatch);

#[test]
fn test_tuple_field() {
    let holder = TupleHolder {
        id: 42,
        data: (100, "hello".to_string()),
        active: true,
    };

    assert_struct!(
        holder,
        TupleHolder {
            id: 42,
            data: (100, "hello"),
            active: true,
        }
    );
}

#[test]
fn test_tuple_field_partial() {
    let holder = TupleHolder {
        id: 99,
        data: (200, "world".to_string()),
        active: false,
    };

    assert_struct!(
        holder,
        TupleHolder {
            data: (200, "world"),
            ..
        }
    );
}

error_message_test!("basic_errors/tuple_field_mismatch.rs", tuple_field_mismatch);
