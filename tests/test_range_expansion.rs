use assert_struct::assert_struct;

#[derive(Debug)]
struct Person {
    age: u32,
    score: f64,
}

#[test]
fn test_range() {
    let person = Person { age: 25, score: 85.5 };
    
    assert_struct!(person, Person {
        age: 18..=65,
        score: 0.0..=100.0,
    });
}

#[test]
#[should_panic(expected = "Field `age` not in range")]
fn test_range_failure() {
    let person = Person { age: 70, score: 85.5 };
    
    assert_struct!(person, Person {
        age: 18..=65,
        score: 0.0..=100.0,
    });
}