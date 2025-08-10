use assert_struct::assert_struct;

// Test deeply nested structures with comparisons
#[derive(Debug)]
struct Company {
    name: String,
    location: Location,
}

#[derive(Debug)]
struct Location {
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
        location: Location {
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
        location: Location {
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
        location: Location {
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
        location: Location {
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
        location: Location {
            country: "USA".to_string(),
            office: Office {
                building: "Tower A".to_string(),
                floor: 15,
                rooms: 5,
            },
        },
    };

    assert_struct!(company, Company {
        location: Location {
            office: Office {
                rooms: 10..=20,  // Should fail: 5 is not in range
                ..
            },
            ..
        },
        ..
    });
}