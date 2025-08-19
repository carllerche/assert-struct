// Test nested field access patterns like field.nested.value

use assert_struct::assert_struct;

#[derive(Debug)]
struct Inner {
    value: i32,
    text: String,
}

#[derive(Debug)]
struct Middle {
    inner: Inner,
    count: u32,
}

#[derive(Debug)]
struct Outer {
    middle: Middle,
    enabled: bool,
}

#[test]
fn test_simple_nested_field_access() {
    let data = Outer {
        middle: Middle {
            inner: Inner {
                value: 42,
                text: "hello".to_string(),
            },
            count: 10,
        },
        enabled: true,
    };

    // Test accessing nested field one level deep
    assert_struct!(data, Outer {
        middle.count: 10,
        enabled: true,
        ..
    });
}

#[test]
fn test_deep_nested_field_access() {
    let data = Outer {
        middle: Middle {
            inner: Inner {
                value: 100,
                text: "world".to_string(),
            },
            count: 5,
        },
        enabled: false,
    };

    // Test accessing deeply nested fields
    assert_struct!(data, Outer {
        middle.inner.value: 100,
        middle.inner.text: "world",
        ..
    });
}

#[test]
fn test_nested_field_with_comparison() {
    let data = Outer {
        middle: Middle {
            inner: Inner {
                value: 75,
                text: "test".to_string(),
            },
            count: 20,
        },
        enabled: true,
    };

    // Test nested fields with comparison operators
    assert_struct!(data, Outer {
        middle.inner.value: > 50,
        middle.count: >= 20,
        ..
    });
}

#[test]
fn test_mixed_nested_and_direct_fields() {
    let data = Outer {
        middle: Middle {
            inner: Inner {
                value: 30,
                text: "mixed".to_string(),
            },
            count: 15,
        },
        enabled: true,
    };

    // Mix nested field access with direct field access
    assert_struct!(data, Outer {
        enabled: true,
        middle.inner.value: 30,
        middle.count: < 20,
        ..
    });
}

#[derive(Debug)]
#[allow(dead_code)]
struct Container {
    data: Data,
    id: u32,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Data {
    items: Items,
    name: String,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Items {
    values: Vec<i32>,
    tags: Vec<String>,
}

#[test]
fn test_nested_field_with_method_calls() {
    let container = Container {
        data: Data {
            items: Items {
                values: vec![1, 2, 3, 4, 5],
                tags: vec!["alpha".to_string(), "beta".to_string()],
            },
            name: "test_container".to_string(),
        },
        id: 123,
    };

    // Test nested field access with method calls
    // Note: Method calls on nested fields are not yet supported
    // We'd need to extend the parser to handle data.name.len() syntax
    assert_struct!(
        container,
        Container {
            id: 123,
            // data.name.len(): 14,  // TODO: Not yet supported
            // data.items.values.len(): 5,  // TODO: Not yet supported
            ..
        }
    );
}

#[test]
fn test_partial_nested_matching() {
    let data = Outer {
        middle: Middle {
            inner: Inner {
                value: 99,
                text: "partial".to_string(),
            },
            count: 7,
        },
        enabled: false,
    };

    // Only check some nested fields, ignore others
    assert_struct!(data, Outer {
        middle.inner.value: 99,
        ..  // Ignore everything else
    });
}

// Complex nested structure for comprehensive testing
#[derive(Debug)]
#[allow(dead_code)]
struct Company {
    info: CompanyInfo,
    departments: Vec<Department>,
}

#[derive(Debug)]
struct CompanyInfo {
    name: String,
    address: Address,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Address {
    street: String,
    city: String,
    zip: u32,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Department {
    name: String,
    manager: Person,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Person {
    name: String,
    age: u32,
}

#[test]
fn test_very_complex_nested_access() {
    let company = Company {
        info: CompanyInfo {
            name: "TechCorp".to_string(),
            address: Address {
                street: "123 Main St".to_string(),
                city: "San Francisco".to_string(),
                zip: 94105,
            },
        },
        departments: vec![Department {
            name: "Engineering".to_string(),
            manager: Person {
                name: "Alice".to_string(),
                age: 35,
            },
        }],
    };

    // Test complex nested field patterns
    assert_struct!(company, Company {
        info.name: "TechCorp",
        info.address.city: "San Francisco",
        info.address.zip: > 90000,
        ..
    });
}

// Error message tests
#[macro_use]
mod util;

error_message_test!(
    "nested_field_access_errors/simple_nested_mismatch.rs",
    simple_nested_mismatch
);

error_message_test!(
    "nested_field_access_errors/deep_nested_mismatch.rs",
    deep_nested_mismatch
);

error_message_test!(
    "nested_field_access_errors/nested_comparison_failure.rs",
    nested_comparison_failure
);
