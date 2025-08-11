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
fn test_enum_variant_mismatch_error() {
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

    println!("Error message:\n{}", message);

    // Check that it's using the fancy format now
    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("   | Status :: Active {"));
    assert!(message.contains("enum variant mismatch"));
    assert!(message.contains("  --> `status` (line 32)"));
    assert!(message.contains("   | Status :: Active {"));
    assert!(message.contains("   | ^^^^^^^^^^^^^^^^ actual: Inactive"));
    assert!(message.contains("   | }"));
}
