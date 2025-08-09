//! Procedural macro implementation for assert-struct.
//!
//! This crate provides the procedural macro implementation for the `assert-struct` crate.
//! Users should use the main `assert-struct` crate which re-exports this macro.
//!
//! See the main `assert-struct` crate for documentation and examples.

use proc_macro::TokenStream;
use syn::{Expr, Token, punctuated::Punctuated};

mod expand;
mod parse;

// Root-level struct that tracks the assertion
struct AssertStruct {
    value: Expr,
    pattern: Pattern,
}

// Unified pattern type that can represent any pattern
enum Pattern {
    // Simple value: 42, "hello", true
    Simple(Expr),
    // Struct pattern: User { name: "Alice", age: 30, .. }
    Struct {
        path: syn::Path,
        fields: Punctuated<FieldAssertion, Token![,]>,
        rest: bool,
    },
    // Tuple pattern: (10, 20) or Some(42) or None
    Tuple {
        path: Option<syn::Path>,
        elements: Vec<Pattern>,
    },
    // Slice pattern: [1, 2, 3] or [1, .., 5]
    Slice(Vec<Pattern>),
    // Comparison: > 30, <= 100
    Comparison(ComparisonOp, Expr),
    // Range: 10..20, 0..=100
    Range(Expr),
    // Regex: =~ r"pattern"
    #[cfg(feature = "regex")]
    Regex(String),
    // Rest pattern: .. for partial matching
    Rest,
}

struct Expected {
    fields: Punctuated<FieldAssertion, Token![,]>,
    rest: bool, // true if ".." was present
}

// Field assertion - a field name paired with its expected pattern
struct FieldAssertion {
    field_name: syn::Ident,
    pattern: Pattern,
}

#[derive(Clone, Copy)]
enum ComparisonOp {
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
}

/// Asserts that a struct matches an expected pattern.
///
/// See the main `assert-struct` crate for full documentation.
#[proc_macro]
pub fn assert_struct(input: TokenStream) -> TokenStream {
    // Parse the input
    let assert = match parse::parse(input) {
        Ok(assert) => assert,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    // Expand to output code
    let expanded = expand::expand(&assert);

    TokenStream::from(expanded)
}