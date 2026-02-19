#![allow(dead_code)]
use assert_struct::assert_struct;

#[macro_use]
mod util;

// ── Basic exact matching ──────────────────────────────────────────────────────

#[test]
fn test_set_exact_match() {
    let items = vec![3, 1, 2];
    assert_struct!(items, #(1, 2, 3));
}

#[test]
fn test_set_exact_match_different_order() {
    let items = vec![3, 1, 2];
    assert_struct!(items, #(3, 1, 2));
    assert_struct!(items, #(2, 3, 1));
}

#[test]
fn test_set_empty() {
    let items: Vec<i32> = vec![];
    assert_struct!(items, #());
}

#[test]
fn test_set_single_element() {
    let items = vec![42];
    assert_struct!(items, #(42));
}

// ── Rest pattern (..) ─────────────────────────────────────────────────────────

#[test]
fn test_set_rest_superset() {
    let items = vec![1, 2, 3, 4, 5];
    assert_struct!(items, #(1, 2, ..));
    assert_struct!(items, #(5, 1, ..));
}

#[test]
fn test_set_rest_exact_size() {
    // rest allows exactly the right size too
    let items = vec![1, 2];
    assert_struct!(items, #(1, 2, ..));
}

#[test]
fn test_set_rest_only() {
    // #(..) matches any collection regardless of contents
    let items = vec![1, 2, 3];
    assert_struct!(items, #(..));
    let empty: Vec<i32> = vec![];
    assert_struct!(empty, #(..));
}

// ── Comparison patterns inside #() ───────────────────────────────────────────

#[test]
fn test_set_comparisons() {
    let scores = vec![95, 42, 77];
    assert_struct!(scores, #(> 90, < 50, ..));
    assert_struct!(scores, #(> 40, > 70, > 90));
}

#[test]
fn test_set_ranges() {
    let ages = vec![25, 45, 60];
    assert_struct!(ages, #(18..=30, 31..=50, 51..=65));
}

// ── Struct patterns inside #() ────────────────────────────────────────────────

#[derive(Debug)]
struct Event {
    kind: String,
    value: i32,
}

#[test]
fn test_set_struct_patterns() {
    let events = vec![
        Event { kind: "click".to_string(), value: 10 },
        Event { kind: "hover".to_string(), value: 20 },
        Event { kind: "scroll".to_string(), value: 30 },
    ];

    assert_struct!(events, #(
        _ { kind: "hover", .. },
        _ { kind: "click", .. },
        _ { kind: "scroll", .. },
    ));
}

#[test]
fn test_set_struct_patterns_with_rest() {
    let events = vec![
        Event { kind: "click".to_string(), value: 10 },
        Event { kind: "hover".to_string(), value: 20 },
        Event { kind: "scroll".to_string(), value: 30 },
    ];

    // Only check that click and hover are present; scroll is ignored
    assert_struct!(events, #(
        _ { kind: "click", .. },
        _ { kind: "hover", .. },
        ..
    ));
}

// ── Backtracking correctness ──────────────────────────────────────────────────

#[test]
fn test_set_backtracking_required() {
    // Greedy would fail: > 5 claims 10, then == 10 fails on 7.
    // Backtracking reassigns: > 5 → 7, == 10 → 10.
    let items = vec![10, 7];
    assert_struct!(items, #(> 5, == 10));
}

#[test]
fn test_set_backtracking_overlapping_patterns() {
    // All elements satisfy > 0; backtracking must assign distinctly
    let items = vec![1, 2, 3];
    assert_struct!(items, #(> 0, > 0, > 0));
}

// ── Enum patterns inside #() ──────────────────────────────────────────────────

#[test]
fn test_set_enum_patterns() {
    let results: Vec<Option<i32>> = vec![Some(5), None, Some(10)];
    assert_struct!(results, #(None, Some(> 0), Some(> 8)));
}

// ── As a struct field ─────────────────────────────────────────────────────────

#[derive(Debug)]
struct Container {
    tags: Vec<String>,
    scores: Vec<i32>,
}

#[test]
fn test_set_as_field() {
    let c = Container {
        tags: vec!["rust".to_string(), "async".to_string(), "test".to_string()],
        scores: vec![85, 92, 78],
    };

    assert_struct!(c, Container {
        tags: #("rust", "async", ..),
        scores: #(> 70, > 80, > 90),
        ..
    });
}

// ── String literals inside #() ────────────────────────────────────────────────

#[test]
fn test_set_string_literals() {
    let words = vec!["hello".to_string(), "world".to_string()];
    assert_struct!(words, #("world", "hello"));
}

// ── Failure cases ─────────────────────────────────────────────────────────────

error_message_test!("sets_errors/exact_wrong_length.rs", test_set_exact_wrong_length);
error_message_test!("sets_errors/exact_too_many_patterns.rs", test_set_exact_too_many_patterns);
error_message_test!("sets_errors/rest_too_few_elements.rs", test_set_rest_too_few_elements);
error_message_test!("sets_errors/no_valid_assignment.rs", test_set_no_valid_assignment);
error_message_test!("sets_errors/empty_mismatch.rs", test_set_empty_mismatch);
