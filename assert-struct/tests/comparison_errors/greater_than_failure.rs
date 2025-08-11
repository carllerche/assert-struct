use assert_struct::assert_struct;

#[derive(Debug)]
struct Person {
    name: String,
    age: u32,
    height: f64,
    score: i32,
}

pub fn test_case() {
    let person = Person {
        name: "Frank".to_string(),
        age: 15,
        height: 5.0,
        score: 40,
    };

    assert_struct!(
        person,
        Person {
            age: > 18,  // This should fail
            ..
        }
    );
}