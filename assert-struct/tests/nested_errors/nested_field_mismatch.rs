#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Address {
    street: String,
    city: String,
    zip: u32,
}

#[derive(Debug)]
struct Person {
    name: String,
    age: u32,
    address: Address,
}

pub fn test_case() {
    let person = Person {
        name: "Charlie".to_string(),
        age: 35,
        address: Address {
            street: "789 Elm St".to_string(),
            city: "Capital City".to_string(),
            zip: 99999,
        },
    };
    assert_struct!(
        person,
        Person {
            name: "Charlie",
            age: 35,
            address: Address {
                street: "789 Elm St",
                city: "Wrong City",
                zip: 99999,
            },
        }
    );
}