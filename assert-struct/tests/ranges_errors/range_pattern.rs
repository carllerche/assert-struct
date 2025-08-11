use assert_struct::assert_struct;

#[derive(Debug)]
struct Person {
    age: u32,
}

pub fn test_case() {
    let person = Person { age: 75 };

    assert_struct!(person, Person {
        age: 18..=65,
    });
}