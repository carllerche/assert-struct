use assert_struct::assert_struct;

#[macro_use]
mod util;

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

#[test]
fn test_nested_struct_exhaustive() {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        address: Address {
            street: "123 Main St".to_string(),
            city: "Springfield".to_string(),
            zip: 12345,
        },
    };

    assert_struct!(
        person,
        Person {
            name: "Alice",
            age: 30,
            address: Address {
                street: "123 Main St",
                city: "Springfield",
                zip: 12345,
            },
        }
    );
}

#[test]
fn test_nested_struct_partial() {
    let person = Person {
        name: "Bob".to_string(),
        age: 25,
        address: Address {
            street: "456 Oak Ave".to_string(),
            city: "Shelbyville".to_string(),
            zip: 54321,
        },
    };

    // Check only some fields of the nested struct
    assert_struct!(
        person,
        Person {
            name: "Bob",
            address: Address {
                city: "Shelbyville",
                ..
            },
            ..
        }
    );
}

#[test]
#[should_panic(expected = "person.address.city")]
fn test_nested_field_mismatch() {
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

error_message_test!("nested_errors/nested_comparison.rs", nested_comparison);

// ============================================================================
// Deeply Nested Structures with Comparisons (merged from nested_deep.rs)
// ============================================================================

#[derive(Debug)]
struct Company {
    name: String,
    location: CompanyLocation,
}

#[derive(Debug)]
struct CompanyLocation {
    country: String,
    office: Office,
}

#[derive(Debug)]
struct Office {
    building: String,
    floor: u32,
    rooms: u32,
}

#[test]
fn test_deeply_nested_with_comparisons() {
    let company = Company {
        name: "TechCorp".to_string(),
        location: CompanyLocation {
            country: "USA".to_string(),
            office: Office {
                building: "Tower A".to_string(),
                floor: 15,
                rooms: 25,
            },
        },
    };

    assert_struct!(company, Company {
        name: "TechCorp",
        location: CompanyLocation {
            country: "USA",
            office: Office {
                building: "Tower A",
                floor: > 10,
                rooms: >= 20,
            },
        },
    });
}

#[test]
#[should_panic(expected = "company.location.office.floor")]
fn test_deeply_nested_comparison_failure() {
    let company = Company {
        name: "TechCorp".to_string(),
        location: CompanyLocation {
            country: "USA".to_string(),
            office: Office {
                building: "Tower A".to_string(),
                floor: 5,
                rooms: 25,
            },
        },
    };

    assert_struct!(company, Company {
        name: "TechCorp",
        location: CompanyLocation {
            country: "USA",
            office: Office {
                building: "Tower A",
                floor: > 10,  // Should fail: 5 is not > 10
                rooms: >= 20,
            },
        },
    });
}

#[test]
#[should_panic(expected = "company.location.office.rooms")]
fn test_deeply_nested_range_failure() {
    let company = Company {
        name: "TechCorp".to_string(),
        location: CompanyLocation {
            country: "USA".to_string(),
            office: Office {
                building: "Tower A".to_string(),
                floor: 15,
                rooms: 5,
            },
        },
    };

    assert_struct!(
        company,
        Company {
            location: CompanyLocation {
                office: Office {
                    rooms: 10..=20, // Should fail: 5 is not in range
                    ..
                },
                ..
            },
            ..
        }
    );
}
