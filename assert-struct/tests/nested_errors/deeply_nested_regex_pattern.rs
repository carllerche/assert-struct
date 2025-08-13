use assert_struct::assert_struct;

#[derive(Debug)]
struct Validation {
    email: String,
    phone: String,
}

#[derive(Debug)]
struct Contact {
    name: String,
    validation: Validation,
}

#[derive(Debug)]
struct Customer {
    id: u64,
    contact: Contact,
}

pub fn test_case() {
    let customer = Customer {
        id: 999,
        contact: Contact {
            name: "John Doe".to_string(),
            validation: Validation {
                email: "john@example.com".to_string(),
                phone: "+1-555-1234".to_string(),
            },
        },
    };

    assert_struct!(customer, Customer {
        id: 999,
        contact: Contact {
            name: "John Doe",
            validation: Validation {
                email: =~ r".*@company\.com$",  // Line 38 - should report this line
                phone: "+1-555-1234",
            },
        },
    });
}