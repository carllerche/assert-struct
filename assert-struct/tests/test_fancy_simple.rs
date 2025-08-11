use assert_struct::assert_struct;
use std::panic;

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
fn test_simple_fancy_format() {
    #[derive(Debug)]
    struct User {
        name: String,
    }

    let user = User {
        name: "Alice".to_string(),
    };

    let message = get_panic_message(|| {
        assert_struct!(user, User { name: "Bob" });
    });

    // Check the exact fancy format from ERRORS.md Example 1
    assert!(message.contains("assert_struct! failed:"));
    assert!(message.contains("   | User {"));
    assert!(message.contains("value mismatch:"));
    assert!(message.contains("  --> `user.name` (line 32)"));
    assert!(message.contains("   |     name: \"Bob\","));
    assert!(message.contains("   |           ^^^^^ actual: \"Alice\""));
    assert!(message.contains("   | }"));
}
