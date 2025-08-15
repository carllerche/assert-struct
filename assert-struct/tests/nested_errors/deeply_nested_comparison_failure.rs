#![allow(dead_code)]
use assert_struct::assert_struct;

#[derive(Debug)]
struct Office {
    building: String,
    floor: u32,
    rooms: u32,
}

#[derive(Debug)]
struct CompanyLocation {
    country: String,
    office: Office,
}

#[derive(Debug)]
struct Company {
    name: String,
    location: CompanyLocation,
}

pub fn test_case() {
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