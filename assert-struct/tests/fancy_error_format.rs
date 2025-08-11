use assert_struct::assert_struct;
use std::panic;

// Helper function to capture panic message
fn get_panic_message<F: FnOnce() + panic::UnwindSafe>(f: F) -> String {
    let result = panic::catch_unwind(f);
    match result {
        Err(err) => {
            if let Some(s) = err.downcast_ref::<String>() {
                s.clone()
            } else if let Some(s) = err.downcast_ref::<&str>() {
                s.to_string()
            } else {
                "Unknown panic message".to_string()
            }
        }
        Ok(_) => panic!("Expected panic but none occurred"),
    }
}

#[test]
fn test_example_1_basic_field_mismatch() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct User {
        name: String,
        age: u32,
    }

    let user = User {
        name: "Alice".to_string(),
        age: 18,
    };

    let message = get_panic_message(|| {
        assert_struct!(user, User { name: "Bob", .. });
    });

    println!("=== Example 1: Basic field mismatch ===");
    println!("{}", message);
    println!("=== Expected from ERRORS.md ===");
    println!(
        r#"assert_struct! failed:

   | User {{
string pattern mismatch:
  --> `user.name` (line 54)
   |     name: "Bob",
   |           ^^^^^ actual: "Alice"
   |     ..
   | }}"#
    );

    // For now, just check that we have the basic structure
    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("user.name"));
}

#[test]
fn test_example_3_nested_field_mismatch() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Profile {
        age: u32,
        verified: bool,
    }

    #[derive(Debug)]
    #[allow(dead_code)]
    struct User {
        name: String,
        profile: Profile,
    }

    let user = User {
        name: "Alice".to_string(),
        profile: Profile {
            age: 17,
            verified: true,
        },
    };

    let message = get_panic_message(|| {
        assert_struct!(user, User {
            profile: Profile {
                age: >= 18,
                ..
            },
            ..
        });
    });

    println!("=== Example 3: Nested field mismatch ===");
    println!("{}", message);
    println!("=== Expected from ERRORS.md ===");
    println!(
        r#"assert_struct! failed:

   | User {{ ... Profile {{
comparison mismatch:
  --> `user.profile.age` (line 136)
   |     age: >= 18,
   |          ^^^^^ actual: 17
   |     ..
   | }} ... }}"#
    );

    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("user.profile.age"));
}

#[test]
fn test_example_4_enum_variant_mismatch() {
    #[derive(Debug, PartialEq)]
    #[allow(dead_code)]
    enum Status {
        Active { since: u32 },
        Inactive,
    }

    let status = Status::Inactive;

    let message = get_panic_message(|| {
        assert_struct!(status, Status::Active { since: > 0 });
    });

    println!("=== Example 4: Enum variant mismatch ===");
    println!("{}", message);
    println!("=== Expected from ERRORS.md ===");
    println!(
        r#"assert_struct! failed:

enum variant mismatch:
  --> `status` (line 171)
   | Status::Active {{ since: > 0 }}
   | ^^^^^^^^^^^^^^ actual: Status::Inactive"#
    );

    // Check the fancy format for enum variant mismatches
    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("enum variant mismatch"));
    assert!(message.contains("Status :: Active"));
    assert!(message.contains("actual: Inactive"));
}
