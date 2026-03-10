/// Tests for wildcard struct patterns that avoid type imports
use assert_struct::assert_struct;

#[macro_use]
mod util;

#[derive(Debug)]
struct Inner {
    value: i32,
    text: String,
}

#[derive(Debug)]
struct Outer {
    inner: Inner,
    count: u32,
}

#[derive(Debug)]
struct Complex {
    outer: Outer,
    enabled: bool,
}

#[test]
fn test_wildcard_struct_simple() {
    let data = Outer {
        inner: Inner {
            value: 42,
            text: "hello".to_string(),
        },
        count: 5,
    };

    // Using wildcard pattern - no need to import Inner type, .. is not required
    assert_struct!(data, _ {
        inner: _ {
            value: 42,
            text: "hello",
        },
        count: 5,
    });
}

#[test]
fn test_wildcard_struct_nested() {
    let data = Complex {
        outer: Outer {
            inner: Inner {
                value: 100,
                text: "world".to_string(),
            },
            count: 10,
        },
        enabled: true,
    };

    // Deep nesting without any type imports
    assert_struct!(data, _ {
        outer: _ {
            inner: _ {
                value: > 50,
                text: "world",
            },
            count: >= 10,
        },
        enabled: true,
    });
}

#[test]
fn test_wildcard_with_comparisons() {
    let data = Outer {
        inner: Inner {
            value: 25,
            text: "test".to_string(),
        },
        count: 8,
    };

    assert_struct!(data, _ {
        inner: _ {
            value: > 20,
            text: != "other",
        },
        count: < 10,
    });
}

#[test]
fn test_wildcard_with_method_calls() {
    let data = Outer {
        inner: Inner {
            value: 0,
            text: "hello world".to_string(),
        },
        count: 3,
    };

    assert_struct!(data, _ {
        inner: _ {
            text.len(): 11,
            text.contains("world"): true,
        },
        count: > 0,
    });
}

#[test]
fn test_wildcard_partial_matching() {
    let data = Complex {
        outer: Outer {
            inner: Inner {
                value: 42,
                text: "ignored".to_string(),
            },
            count: 99,
        },
        enabled: false,
    };

    // Only check specific fields, ignore the rest (.. is implicit for _ patterns)
    assert_struct!(data, _ {
        outer: _ {
            inner: _ {
                value: 42,
            },
        },
    });
}

error_message_test!(
    "wildcard_errors/wildcard_struct_failure.rs",
    wildcard_struct_failure
);

#[test]
fn test_wildcard_with_options() {
    #[derive(Debug)]
    struct Container {
        maybe_inner: Option<Inner>,
    }

    let data = Container {
        maybe_inner: Some(Inner {
            value: 42,
            text: "present".to_string(),
        }),
    };

    // Combine wildcard struct with Option pattern
    assert_struct!(data, _ {
        maybe_inner: Some(_ {
            value: 42,
            text: "present",
        }),
    });
}

// Explicit .. is still accepted in wildcard patterns for those who prefer it
#[test]
fn test_wildcard_explicit_rest_still_works() {
    let data = Outer {
        inner: Inner {
            value: 7,
            text: "ok".to_string(),
        },
        count: 1,
    };

    assert_struct!(data, _ {
        count: 1,
        ..
    });
}

// Bare anonymous struct syntax: { ... } without _ prefix
#[test]
fn test_bare_anonymous_struct_simple() {
    let data = Inner {
        value: 42,
        text: "hello".to_string(),
    };

    assert_struct!(data, {
        value: 42,
        text: "hello",
    });
}

#[test]
fn test_bare_anonymous_struct_nested() {
    let data = Complex {
        outer: Outer {
            inner: Inner {
                value: 100,
                text: "world".to_string(),
            },
            count: 10,
        },
        enabled: true,
    };

    // Nested bare anonymous structs
    assert_struct!(data, {
        outer: {
            inner: {
                value: > 50,
                text: "world",
            },
            count: >= 10,
        },
        enabled: true,
    });
}

#[test]
fn test_bare_anonymous_struct_partial() {
    let data = Complex {
        outer: Outer {
            inner: Inner {
                value: 42,
                text: "ignored".to_string(),
            },
            count: 99,
        },
        enabled: false,
    };

    // Partial matching is implicit (like _ { ... })
    assert_struct!(data, {
        outer: {
            inner: {
                value: 42,
            },
        },
    });
}

#[test]
fn test_bare_anonymous_struct_with_option() {
    #[derive(Debug)]
    struct Container {
        maybe_inner: Option<Inner>,
    }

    let data = Container {
        maybe_inner: Some(Inner {
            value: 42,
            text: "present".to_string(),
        }),
    };

    assert_struct!(data, {
        maybe_inner: Some({
            value: 42,
            text: "present",
        }),
    });
}

#[test]
fn test_bare_anonymous_struct_with_explicit_rest() {
    let data = Outer {
        inner: Inner {
            value: 7,
            text: "ok".to_string(),
        },
        count: 1,
    };

    assert_struct!(data, {
        count: 1,
        ..
    });
}

// This test verifies that normal struct patterns still work
#[test]
fn test_normal_struct_pattern_still_works() {
    let data = Inner {
        value: 100,
        text: "normal".to_string(),
    };

    // Traditional pattern with type name
    assert_struct!(
        data,
        Inner {
            value: 100,
            text: "normal",
        }
    );
}
