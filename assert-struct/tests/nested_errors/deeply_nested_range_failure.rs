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