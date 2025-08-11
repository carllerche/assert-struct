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
fn test_equality_pattern_expression() {
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

    let expected_name = "Bob";

    let message = get_panic_message(|| {
        assert_struct!(user, User {
            name: == expected_name,
            age: 18,
        });
    });

    println!("Error message:\n{}", message);

    // Verify fancy format with expression underlined
    assert!(message.contains("assert_struct! failed"));
    assert!(message.contains("   | User {"));
    assert!(message.contains("equality mismatch:"));
    assert!(message.contains("   |     name: == expected_name,"));

    // The expression (not the operator) should be underlined
    assert!(message.contains("expected_name"));
    assert!(message.contains("actual: \"Alice\""));
    assert!(message.contains("expected: \"Bob\""));
}

#[test]
fn test_equality_pattern_with_variable() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Config {
        setting: String,
        timeout: u32,
    }

    let config = Config {
        setting: "production".to_string(),
        timeout: 30,
    };

    let expected_setting = "development";

    let message = get_panic_message(|| {
        assert_struct!(config, Config {
            setting: == expected_setting,
            timeout: 30,
        });
    });

    println!("Error message:\n{}", message);

    // Check that it shows both actual and expected values
    assert!(message.contains("actual: \"production\""));
    assert!(message.contains("expected: \"development\""));
}

#[test]
fn test_inequality_pattern() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct Data {
        value: u32,
        flag: bool,
    }

    let data = Data {
        value: 100,
        flag: true,
    };

    let message = get_panic_message(|| {
        assert_struct!(data, Data {
            value: != 100,  // This should fail since value is 100
            flag: true,
        });
    });

    println!("Error message:\n{}", message);

    assert!(message.contains("assert_struct! failed"));
    // For != patterns, we show it as comparison, not equality
    assert!(message.contains("comparison mismatch:"));
    assert!(message.contains("   |     value: != 100,"));
    assert!(message.contains("actual: 100"));
}
