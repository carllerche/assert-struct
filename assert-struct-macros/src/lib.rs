//! Procedural macro implementation for assert-struct.
//!
//! This crate provides the procedural macro implementation for the `assert-struct` crate.
//! Users should use the main `assert-struct` crate which re-exports this macro.
//!
//! # Architecture Overview
//!
//! The macro transformation happens in three phases:
//!
//! 1. **Parse** (`parse.rs`): Tokenize the macro input into a Pattern AST
//! 2. **Expand** (`expand.rs`): Transform patterns into assertion code
//! 3. **Execute**: Generated code runs the actual assertions
//!
//! # Key Design Decisions
//!
//! - **Pattern enum**: Unified abstraction for all pattern types (struct, tuple, slice, etc.)
//! - **Disambiguation**: `check_for_special_syntax` solves `Some(> 30)` vs `Some(my_var)`
//! - **Dual-path optimization**: String literal regexes compile at expansion time
//! - **Native Rust syntax**: Use match expressions for ranges, slices, and enums
//!
//! See the main `assert-struct` crate for documentation and examples.

use proc_macro::TokenStream;
use std::fmt;
use syn::{Expr, Token, punctuated::Punctuated};

mod expand;
mod parse;

// Root-level struct that tracks the assertion
struct AssertStruct {
    value: Expr,
    pattern: Pattern,
}

// Unified pattern type that can represent any pattern
pub(crate) enum Pattern {
    // Simple value: 42, "hello", true
    Simple {
        node_id: usize,
        expr: Expr,
    },
    // Struct pattern: User { name: "Alice", age: 30, .. }
    Struct {
        node_id: usize,
        path: syn::Path,
        fields: Punctuated<FieldAssertion, Token![,]>,
        rest: bool,
    },
    // Tuple pattern: (10, 20) or Some(42) or None
    Tuple {
        node_id: usize,
        path: Option<syn::Path>,
        elements: Vec<Pattern>,
    },
    // Slice pattern: [1, 2, 3] or [1, .., 5]
    Slice {
        node_id: usize,
        elements: Vec<Pattern>,
    },
    // Comparison: > 30, <= 100
    Comparison {
        node_id: usize,
        op: ComparisonOp,
        expr: Expr,
    },
    // Range: 10..20, 0..=100
    Range {
        node_id: usize,
        expr: Expr,
    },
    // Regex: =~ "pattern" - string literal optimized at compile time
    #[cfg(feature = "regex")]
    Regex {
        node_id: usize,
        pattern: String, // String literal regex pattern (performance optimization)
        span: proc_macro2::Span, // Store span for accurate error reporting
    },
    // Like pattern: =~ expr - arbitrary expression using Like trait
    #[cfg(feature = "regex")]
    Like {
        node_id: usize,
        expr: Expr,
    },
    // Rest pattern: .. for partial matching
    Rest {
        node_id: usize,
    },
    // Wildcard pattern: _ for ignoring a value while asserting it exists
    Wildcard {
        node_id: usize,
    },
}

// Helper function to format syn expressions as strings
fn expr_to_string(expr: &Expr) -> String {
    // This is a simplified version - in production we'd want more complete handling
    match expr {
        Expr::Lit(lit) => {
            // Handle literals
            quote::quote! { #lit }.to_string()
        }
        Expr::Path(path) => {
            // Handle paths
            quote::quote! { #path }.to_string()
        }
        Expr::Range(range) => {
            // Handle ranges
            quote::quote! { #range }.to_string()
        }
        _ => {
            // Fallback - use quote for other expressions
            quote::quote! { #expr }.to_string()
        }
    }
}

fn path_to_string(path: &syn::Path) -> String {
    quote::quote! { #path }.to_string()
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Simple { expr, .. } => {
                write!(f, "{}", expr_to_string(expr))
            }
            Pattern::Struct {
                path, fields, rest, ..
            } => {
                write!(f, "{} {{ ", path_to_string(path))?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field.field_name, field.pattern)?;
                }
                if *rest {
                    if !fields.is_empty() {
                        write!(f, ", ")?;
                    }
                    write!(f, "..")?;
                }
                write!(f, " }}")
            }
            Pattern::Tuple { path, elements, .. } => {
                if let Some(p) = path {
                    write!(f, "{}", path_to_string(p))?;
                }
                write!(f, "(")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, ")")
            }
            Pattern::Slice { elements, .. } => {
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
            Pattern::Comparison { op, expr, .. } => {
                write!(f, "{} {}", op, expr_to_string(expr))
            }
            Pattern::Range { expr, .. } => {
                write!(f, "{}", expr_to_string(expr))
            }
            #[cfg(feature = "regex")]
            Pattern::Regex { pattern, .. } => {
                write!(f, "=~ r\"{}\"", pattern)
            }
            #[cfg(feature = "regex")]
            Pattern::Like { expr, .. } => {
                write!(f, "=~ {}", expr_to_string(expr))
            }
            Pattern::Rest { .. } => {
                write!(f, "..")
            }
            Pattern::Wildcard { .. } => {
                write!(f, "_")
            }
        }
    }
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
pub(crate) enum ComparisonOp {
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
}

impl fmt::Display for ComparisonOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComparisonOp::Less => write!(f, "<"),
            ComparisonOp::LessEqual => write!(f, "<="),
            ComparisonOp::Greater => write!(f, ">"),
            ComparisonOp::GreaterEqual => write!(f, ">="),
            ComparisonOp::Equal => write!(f, "=="),
            ComparisonOp::NotEqual => write!(f, "!="),
        }
    }
}

/// Asserts that a struct matches an expected pattern.
///
/// This macro transforms structural patterns into runtime assertions, generating
/// efficient code that provides clear error messages when assertions fail.
///
/// See the [crate-level documentation](crate) for a comprehensive guide and examples.
///
/// # Syntax
///
/// ```text
/// assert_struct!(value, TypeName {
///     field: matcher,
///     ...
/// });
/// ```
///
/// # Matchers
///
/// | Matcher | Description | Example |
/// |---------|-------------|---------|
/// | Exact value | Direct equality comparison | `name: "Alice"` |
/// | Equality | Explicit equality/inequality | `age: == 30`, `status: != "error"` |
/// | Comparison | Numeric comparisons | `age: >= 18` |
/// | Range | Match values in ranges | `age: 18..=65`, `score: 0.0..100.0` |
/// | Regex | Pattern matching (requires `regex` feature) | `email: =~ r"@.*\.com$"` |
/// | Option | Match `Some` and `None` variants | `age: Some(30)`, `bio: None` |
/// | Result | Match `Ok` and `Err` variants | `result: Ok(200)`, `error: Err("failed")` |
/// | Custom enum | Match custom enum variants | `status: Status::Active` |
/// | Nested struct | Recursive structural matching | `address: Address { city: "Boston", .. }` |
/// | Tuple | Element-wise comparison | `point: (10, 20)` |
/// | Vec/slice | Element-wise patterns | `items: [1, 2, 3]` or `items: [> 0, < 10, == 5]` |
/// | Partial | Ignore remaining fields | `..` |
///
/// # Parameters
///
/// - **`value`**: The struct instance to test
/// - **`TypeName`**: The struct type (must match the value's type)
/// - **`{ fields }`**: Pattern specifying expected field values
///
/// # Examples
///
/// ## Basic Usage
///
/// ```
/// # use assert_struct::assert_struct;
/// # #[derive(Debug)]
/// # struct User { name: String, age: u32 }
/// # let user = User { name: "Alice".to_string(), age: 30 };
/// assert_struct!(user, User {
///     name: "Alice",
///     age: 30,
/// });
/// ```
///
/// ## Partial Matching
///
/// ```
/// # use assert_struct::assert_struct;
/// # #[derive(Debug)]
/// # struct User { name: String, age: u32, email: String }
/// # let user = User { name: "Bob".to_string(), age: 25, email: "bob@example.com".to_string() };
/// assert_struct!(user, User {
///     name: "Bob",
///     ..  // Ignores age and email
/// });
/// ```
///
/// ## Comparison and Equality Operators
///
/// ```
/// # use assert_struct::assert_struct;
/// # #[derive(Debug, PartialEq)]
/// # struct Score { value: i32, bonus: f64, grade: String }
/// # let score = Score { value: 95, bonus: 1.5, grade: "A".to_string() };
/// assert_struct!(score, Score {
///     value: > 90,        // Greater than
///     bonus: >= 1.0,      // Greater or equal
///     grade: == "A",      // Explicit equality
/// });
///
/// assert_struct!(score, Score {
///     grade: != "F",      // Not equal
///     ..
/// });
/// ```
///
/// ## Range Patterns
///
/// ```
/// # use assert_struct::assert_struct;
/// # #[derive(Debug)]
/// # struct Person { age: u32, score: f64, level: i32 }
/// # let person = Person { age: 25, score: 85.5, level: 10 };
/// assert_struct!(person, Person {
///     age: 18..=65,       // Inclusive range
///     score: 0.0..100.0,  // Exclusive range
///     level: 0..,         // Range from (unbounded end)
/// });
/// ```
///
/// ## Complex Expressions
///
/// ```
/// # use assert_struct::assert_struct;
/// # #[derive(Debug)]
/// # struct Metrics {
/// #     cpu_usage: f64,
/// #     memory_mb: u32,
/// #     response_time_ms: u32,
/// # }
/// # struct Config { min_memory: u32 }
/// # fn get_threshold() -> f64 { 75.0 }
/// # let config = Config { min_memory: 512 };
/// # let limits = [100, 200, 300];
/// # let metrics = Metrics {
/// #     cpu_usage: 70.0,
/// #     memory_mb: 1024,
/// #     response_time_ms: 150,
/// # };
/// assert_struct!(metrics, Metrics {
///     cpu_usage: < get_threshold() + 5.0,  // Function calls with arithmetic
///     memory_mb: >= config.min_memory,     // Field access
///     response_time_ms: < limits[2],       // Array indexing
/// });
/// ```
///
/// ## Regex Patterns
///
/// ```
/// # #[cfg(feature = "regex")]
/// # {
/// # use assert_struct::assert_struct;
/// # #[derive(Debug)]
/// # struct User { email: String }
/// # let user = User { email: "test@example.com".to_string() };
/// assert_struct!(user, User {
///     email: =~ r"^[^@]+@[^@]+\.[^@]+$",
/// });
/// # }
/// ```
///
/// ## Enum Support
///
/// ```
/// # use assert_struct::assert_struct;
/// # #[derive(Debug)]
/// # struct Config {
/// #     timeout: Option<u32>,
/// #     retry_count: Option<u32>,
/// #     result: Result<String, String>,
/// # }
/// # let config = Config {
/// #     timeout: Some(5000),
/// #     retry_count: None,
/// #     result: Ok("success".to_string()),
/// # };
/// assert_struct!(config, Config {
///     timeout: Some(> 1000),  // Comparison inside Some
///     retry_count: None,
///     result: Ok("success"),
/// });
/// ```
///
/// ## Tuples
///
/// ```
/// # use assert_struct::assert_struct;
/// # #[derive(Debug)]
/// # struct Data {
/// #     point: (i32, i32),
/// #     triple: (String, u32, bool),
/// # }
/// # let data = Data {
/// #     point: (15, 25),
/// #     triple: ("test".to_string(), 100, true),
/// # };
/// assert_struct!(data, Data {
///     point: (> 10, < 30),  // Comparisons in tuples
///     triple: ("test", >= 50, true),  // Mixed patterns
/// });
/// ```
///
/// # Behavior
///
/// ## Non-consuming
///
/// The macro borrows the value, so it remains available after the assertion:
///
/// ```
/// # use assert_struct::assert_struct;
/// # #[derive(Debug)]
/// # struct Data { value: i32 }
/// let data = Data { value: 42 };
/// assert_struct!(data, Data { value: 42 });
/// println!("{:?}", data);  // data is still available
/// ```
///
/// ## Field Order
///
/// Fields can be specified in any order:
///
/// ```
/// # use assert_struct::assert_struct;
/// # #[derive(Debug)]
/// # struct Point { x: i32, y: i32 }
/// # let point = Point { x: 1, y: 2 };
/// assert_struct!(point, Point {
///     y: 2,  // Order doesn't matter
///     x: 1,
/// });
/// ```
///
/// ## Exhaustive vs Partial
///
/// Without `..`, all fields must be specified:
///
/// ```compile_fail
/// # use assert_struct::assert_struct;
/// # #[derive(Debug)]
/// # struct User { name: String, age: u32 }
/// # let user = User { name: "Alice".to_string(), age: 30 };
/// assert_struct!(user, User {
///     name: "Alice",
///     // Error: missing field `age`
/// });
/// ```
///
/// # Error Messages
///
/// When assertions fail, the macro provides detailed error messages showing exactly what went wrong:
///
/// ```should_panic
/// # use assert_struct::assert_struct;
/// # #[derive(Debug)]
/// # struct User { name: String, age: u32 }
/// # let user = User { name: "Alice".to_string(), age: 25 };
/// assert_struct!(user, User {
///     name: "Bob",  // This assertion will fail
///     age: 25,
/// });
/// // Error output:
/// // assert_struct! failed:
/// //
/// // value mismatch:
/// //   --> `user.name` (src/lib.rs:319)
/// //   actual: "Alice"
/// //   expected: "Bob"
/// ```
///
/// For comparison patterns, the error clearly shows the failed condition:
///
/// ```should_panic
/// # use assert_struct::assert_struct;
/// # #[derive(Debug)]
/// # struct Account { balance: f64 }
/// # let account = Account { balance: 50.0 };
/// assert_struct!(account, Account {
///     balance: > 100.0,  // This will fail
/// });
/// // Error output:
/// // assert_struct! failed:
/// //
/// // comparison mismatch:
/// //   --> `account.balance` (src/lib.rs:334)
/// //   actual: 50.0
/// //   expected: > 100.0
/// ```
///
/// # Compilation Errors
///
/// The macro fails to compile if:
/// - Field names don't exist
/// - Types are incompatible (except with matchers)
/// - Syntax is invalid
///
/// ```compile_fail
/// # use assert_struct::assert_struct;
/// # #[derive(Debug)]
/// # struct User { name: String }
/// # let user = User { name: "Alice".to_string() };
/// assert_struct!(user, User {
///     nonexistent: "value",  // Error: no field named `nonexistent`
/// });
/// ```
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
