use assert_struct::assert_struct;

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

#[test]
#[should_panic]
fn test_field_mismatch() {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
    };

    assert_struct!(
        user,
        User {
            name: "Bob",
            age: 30,
        }
    );
}

#[test]
#[should_panic]
fn test_age_mismatch() {
    let user = User {
        name: "Alice".to_string(),
        age: 30,
    };

    assert_struct!(
        user,
        User {
            name: "Alice",
            age: 25,
        }
    );
}

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

#[test]
#[should_panic]
fn test_vec_mismatch() {
    let data = DataHolder {
        values: vec![1, 2, 3],
        name: "test".to_string(),
    };

    assert_struct!(
        data,
        DataHolder {
            values: &[1, 2, 4], // Wrong last element
            name: "test",
        }
    );
}

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

#[test]
#[should_panic]
fn test_tuple_field_mismatch() {
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
