// Common test utilities

use std::panic;

/// Captures the panic message from a function that should panic
pub fn capture_panic_message<F: FnOnce() + panic::UnwindSafe>(f: F) -> String {
    let result = panic::catch_unwind(f);
    let err = result.unwrap_err();
    err.downcast_ref::<String>()
        .map(|s| s.as_str())
        .or_else(|| err.downcast_ref::<&str>().copied())
        .unwrap()
        .to_string()
}

/// Macro to reduce boilerplate for error message tests
#[macro_export]
macro_rules! error_message_test {
    ($path:literal, $name:ident) => {
        #[path = $path]
        mod $name;

        #[test]
        fn $name() {
            let message = util::capture_panic_message(|| {
                $name::test_case();
            });
            insta::assert_snapshot!(message);
        }
    };
    // Variant with feature gate
    (#[cfg($cfg:meta)] $path:literal, $name:ident) => {
        #[cfg($cfg)]
        #[path = $path]
        mod $name;

        #[test]
        #[cfg($cfg)]
        fn $name() {
            let message = util::capture_panic_message(|| {
                $name::test_case();
            });
            insta::assert_snapshot!(message);
        }
    };
}
