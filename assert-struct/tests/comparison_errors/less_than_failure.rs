use assert_struct::assert_struct;

#[derive(Debug)]
#[allow(dead_code)]
struct Person {
    name: String,
    age: u32,
    height: f64,
    score: i32,
}

pub fn test_case() {
    let person = Person {
        name: "Grace".to_string(),
        age: 25,
        height: 6.2,
        score: 100,
    };

    assert_struct!(
        person,
        Person {
            height: < 6.0,  // This should fail
            ..
        }
    );
}