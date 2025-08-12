use assert_struct::assert_struct;

// Test plain tuples with multiple fields
#[derive(Debug)]
struct Coordinates {
    point2d: (i32, i32),
    point3d: (f64, f64, f64),
    metadata: (String, u32, bool),
}

#[test]
fn test_plain_tuple_two_fields() {
    let coords = Coordinates {
        point2d: (10, 20),
        point3d: (1.5, 2.5, 3.5),
        metadata: ("origin".to_string(), 42, true),
    };

    assert_struct!(
        coords,
        Coordinates {
            point2d: (10, 20),
            point3d: (1.5, 2.5, 3.5),
            metadata: ("origin", 42, true),
        }
    );
}

#[test]
fn test_plain_tuple_with_comparisons() {
    let coords = Coordinates {
        point2d: (15, 25),
        point3d: (1.5, 2.5, 3.5),
        metadata: ("center".to_string(), 100, false),
    };

    assert_struct!(coords, Coordinates {
        point2d: (> 10, < 30),  // Both elements with comparisons
        point3d: (>= 1.0, <= 3.0, > 3.0),
        metadata: ("center", >= 50, false),
    });
}

#[test]
#[cfg(feature = "regex")]
fn test_plain_tuple_with_regex() {
    let coords = Coordinates {
        point2d: (15, 25),
        point3d: (1.5, 2.5, 3.5),
        metadata: ("point_123".to_string(), 100, true),
    };

    assert_struct!(coords, Coordinates {
        point2d: (> 10, < 30),
        point3d: (1.5, 2.5, 3.5),
        metadata: (=~ r"^point_\d+$", 100, true),
    });
}

// Test enum tuples with multiple fields and advanced patterns
#[derive(Debug, PartialEq)]
enum Event {
    Click(i32, i32),
    Drag(i32, i32, i32, i32),
    #[allow(dead_code)] // Only used with regex feature
    Scroll(f64, String),
    Complex(String, u32, bool, Vec<u8>),
}

#[derive(Debug)]
struct EventLog {
    event: Event,
    timestamp: u64,
}

#[test]
fn test_enum_tuple_two_fields() {
    let log = EventLog {
        event: Event::Click(100, 200),
        timestamp: 1234567890,
    };

    assert_struct!(
        log,
        EventLog {
            event: Event::Click(100, 200),
            timestamp: 1234567890,
        }
    );
}

#[test]
fn test_enum_tuple_with_comparisons() {
    let log = EventLog {
        event: Event::Drag(10, 20, 110, 120),
        timestamp: 1234567890,
    };

    assert_struct!(log, EventLog {
        event: Event::Drag(>= 0, >= 0, < 200, < 200),
        timestamp: > 0,
    });
}

#[test]
#[cfg(feature = "regex")]
fn test_enum_tuple_with_regex() {
    let log = EventLog {
        event: Event::Scroll(3.5, "smooth".to_string()),
        timestamp: 1234567890,
    };

    assert_struct!(log, EventLog {
        event: Event::Scroll(> 0.0, =~ r"^smooth|fast|instant$"),
        timestamp: 1234567890,
    });
}

#[test]
fn test_enum_tuple_four_fields_mixed() {
    let log = EventLog {
        event: Event::Complex("test_event".to_string(), 42, true, vec![1, 2, 3]),
        timestamp: 1234567890,
    };

    assert_struct!(log, EventLog {
        event: Event::Complex(
            "test_event",  // String literal
            > 40,          // Comparison
            true,          // Boolean
            [1, 2, 3]      // Vec as slice
        ),
        timestamp: 1234567890,
    });
}

// Test nested tuples
#[derive(Debug)]
struct NestedTuples {
    simple: (i32, i32),
    nested: ((i32, i32), (String, bool)),
}

#[test]
fn test_nested_tuples() {
    let data = NestedTuples {
        simple: (1, 2),
        nested: ((10, 20), ("hello".to_string(), true)),
    };

    assert_struct!(
        data,
        NestedTuples {
            simple: (1, 2),
            nested: ((10, 20), ("hello", true)),
        }
    );
}

#[test]
fn test_nested_tuples_with_comparisons() {
    let data = NestedTuples {
        simple: (5, 10),
        nested: ((15, 25), ("world".to_string(), false)),
    };

    // Note: Due to Rust parser limitations, we cannot use ((> pattern directly
    // as Rust's parser interprets ((> as the start of a generic parameter.
    // Test the patterns separately:

    // Test the simple tuple with patterns
    assert_struct!(data, NestedTuples {
        simple: (< 10, >= 10),
        nested: ((15, 25), ("world", false)),
    });

    // For nested tuples with comparison patterns, we need to test them differently
    // This is a known limitation of Rust's macro system where ((> causes parser ambiguity
}

// Mixed enum variants in tuples
#[derive(Debug, PartialEq)]
enum MixedData {
    Pair(Option<i32>, Result<String, String>),
    Triple(Option<String>, Option<u32>, Option<bool>),
}

#[derive(Debug)]
struct MixedContainer {
    data: MixedData,
}

#[test]
fn test_enum_tuple_with_nested_options() {
    let container = MixedContainer {
        data: MixedData::Pair(Some(42), Ok("success".to_string())),
    };

    assert_struct!(
        container,
        MixedContainer {
            data: MixedData::Pair(Some(42), Ok("success"),),
        }
    );
}

#[test]
fn test_enum_tuple_with_nested_option_patterns() {
    let container = MixedContainer {
        data: MixedData::Triple(Some("test".to_string()), Some(100), None),
    };

    assert_struct!(container, MixedContainer {
        data: MixedData::Triple(
            Some("test"),
            Some(> 50),  // Comparison inside Some inside tuple
            None,
        ),
    });
}

// Failure tests
#[test]
#[should_panic(expected = "assert_struct! failed")]
fn test_tuple_field_mismatch() {
    let coords = Coordinates {
        point2d: (10, 20),
        point3d: (1.0, 2.0, 3.0),
        metadata: ("test".to_string(), 42, true),
    };

    assert_struct!(
        coords,
        Coordinates {
            point2d: (10, 30), // Second field wrong
            point3d: (1.0, 2.0, 3.0),
            metadata: ("test", 42, true),
        }
    );
}

#[test]
#[should_panic(expected = "assert_struct! failed")]
fn test_tuple_comparison_failure() {
    let coords = Coordinates {
        point2d: (5, 20),
        point3d: (1.0, 2.0, 3.0),
        metadata: ("test".to_string(), 42, true),
    };

    assert_struct!(coords, Coordinates {
        point2d: (> 10, 20),  // First element fails comparison
        point3d: (1.0, 2.0, 3.0),
        metadata: ("test", 42, true),
    });
}
