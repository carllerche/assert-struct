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
#[derive(Debug, Clone)]
pub(crate) enum Pattern {
    // Simple value: 42, \"hello\", true
    Simple {
        node_id: usize,
        expr: Expr,
    },
    // Struct pattern: User { name: \"Alice\", age: 30, .. }
    // When path is None, it's a wildcard pattern: _ { name: \"Alice\", .. }
    Struct {
        node_id: usize,
        path: Option<syn::Path>, // None for wildcard patterns
        fields: Punctuated<FieldAssertion, Token![,]>,
        rest: bool,
    },
    // Tuple pattern: (10, 20) or Some(42) or None
    // Now supports mixed positional and indexed elements
    Tuple {
        node_id: usize,
        path: Option<syn::Path>,
        elements: Vec<TupleElement>,
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
    // Closure pattern: |x| expr for custom validation (escape hatch)
    Closure {
        node_id: usize,
        closure: syn::ExprClosure,
    },
    // Map pattern: #{ "key": pattern, .. } for map-like structures
    Map {
        node_id: usize,
        entries: Vec<(syn::Expr, Pattern)>, // key-value pairs
        rest: bool,                         // partial matching with ..
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
                if let Some(p) = path {
                    write!(f, "{} {{ ", path_to_string(p))?;
                } else {
                    write!(f, "_ {{ ")?;
                }
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
                write!(f, r#"=~ r"{}""#, pattern)
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
            Pattern::Closure { closure, .. } => {
                write!(f, "{}", quote::quote! { #closure })
            }
            Pattern::Map { entries, rest, .. } => {
                write!(f, "#{{ ")?;
                for (i, (key, value)) in entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", expr_to_string(key), value)?;
                }
                if *rest {
                    if !entries.is_empty() {
                        write!(f, ", ")?;
                    }
                    write!(f, "..")?;
                }
                write!(f, " }}")
            }
        }
    }
}

struct Expected {
    fields: Punctuated<FieldAssertion, Token![,]>,
    rest: bool, // true if ".." was present
}

/// Represents an operation to be performed on a field before pattern matching
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum FieldOperation {
    /// Dereference operation: *field, **field, etc.
    /// The count indicates how many dereferences to perform
    Deref {
        count: usize,
        span: proc_macro2::Span,
    },

    /// Method call: field.method(), field.len(), etc.
    /// Stores the method name and arguments (if any)
    Method {
        name: syn::Ident,
        args: Vec<syn::Expr>,
        span: proc_macro2::Span,
    },

    /// Await operation: field.await
    /// For async futures that need to be awaited
    Await { span: proc_macro2::Span },

    /// Nested field access: field.nested, field.inner.value, etc.
    /// Stores the chain of field names to access
    Nested {
        fields: Vec<syn::Ident>,
        span: proc_macro2::Span,
    },

    /// Index operation: field\[0\], field\[index\], etc.
    /// Stores the index expression to use
    Index {
        index: syn::Expr,
        span: proc_macro2::Span,
    },

    /// Combined operation: dereferencing followed by method/nested/index access
    /// Example: *field.method(), **field.inner, *field\[0\], etc.
    Combined {
        deref_count: usize,
        operation: Box<FieldOperation>,
        span: proc_macro2::Span,
    },

    /// Chained operations: nested field followed by index or method
    /// Example: field.nested\[0\], field.inner.method(), field.sub\[1\].len()
    Chained {
        operations: Vec<FieldOperation>,
        span: proc_macro2::Span,
    },
}

// Field assertion - a field name paired with its expected pattern
// Now supports operations like dereferencing, method calls, and nested access
#[derive(Debug, Clone)]
struct FieldAssertion {
    field_name: syn::Ident,
    operations: Option<FieldOperation>,
    pattern: Pattern,
}

/// Represents an element in a tuple pattern, supporting both positional and indexed syntax
#[derive(Debug, Clone)]
enum TupleElement {
    /// Positional element: just a pattern in sequence
    /// Example: "foo", > 10, Some(42)
    Positional { pattern: Pattern },

    /// Indexed element: explicit index with optional operations
    /// Example: 0: "foo", *1: "bar", 2.len(): 5
    Indexed {
        index: usize,
        operations: Option<FieldOperation>,
        pattern: Pattern,
    },
}
impl fmt::Display for TupleElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TupleElement::Positional { pattern } => {
                write!(f, "{}", pattern)
            }
            TupleElement::Indexed {
                index,
                operations,
                pattern,
            } => {
                if let Some(ops) = operations {
                    match ops {
                        FieldOperation::Deref { count, .. } => {
                            // Show deref operations before the index: *0:
                            for _ in 0..*count {
                                write!(f, "*")?;
                            }
                            write!(f, "{}: {}", index, pattern)
                        }
                        FieldOperation::Method { name, .. } => {
                            // Show method calls after the index: 0.len():
                            write!(f, "{}.{}(): {}", index, name, pattern)
                        }
                        FieldOperation::Await { .. } => {
                            // Show await after the index: 0.await:
                            write!(f, "{}.await: {}", index, pattern)
                        }
                        FieldOperation::Nested { fields, .. } => {
                            // Show nested access after the index: 0.field:
                            write!(f, "{}", index)?;
                            for field in fields {
                                write!(f, ".{}", field)?;
                            }
                            write!(f, ": {}", pattern)
                        }
                        FieldOperation::Index { index: idx, .. } => {
                            // Show index access after the tuple index: 0[1]:
                            write!(f, "{}[{}]: {}", index, quote::quote! { #idx }, pattern)
                        }
                        FieldOperation::Chained { operations, .. } => {
                            // Show chained operations after the tuple index: 0.field[1]:
                            write!(f, "{}", index)?;
                            for op in operations {
                                match op {
                                    FieldOperation::Nested { fields, .. } => {
                                        for field in fields {
                                            write!(f, ".{}", field)?;
                                        }
                                    }
                                    FieldOperation::Method { name, .. } => {
                                        write!(f, ".{}()", name)?;
                                    }
                                    FieldOperation::Await { .. } => {
                                        write!(f, ".await")?;
                                    }
                                    FieldOperation::Index { index, .. } => {
                                        write!(f, "[{}]", quote::quote! { #index })?;
                                    }
                                    _ => write!(f, "{}", op)?,
                                }
                            }
                            write!(f, ": {}", pattern)
                        }
                        FieldOperation::Combined {
                            deref_count,
                            operation,
                            ..
                        } => {
                            // Show combined operations: *0.len():
                            for _ in 0..*deref_count {
                                write!(f, "*")?;
                            }
                            match operation.as_ref() {
                                FieldOperation::Method { name, .. } => {
                                    write!(f, "{}.{}(): {}", index, name, pattern)
                                }
                                _ => {
                                    write!(f, "{}{}: {}", index, operation, pattern)
                                }
                            }
                        }
                    }
                } else {
                    write!(f, "{}: {}", index, pattern)
                }
            }
        }
    }
}

impl fmt::Display for FieldOperation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldOperation::Deref { count, .. } => {
                for _ in 0..*count {
                    write!(f, "*")?;
                }
                Ok(())
            }
            FieldOperation::Method { name, .. } => {
                write!(f, ".{}()", name)
            }
            FieldOperation::Await { .. } => {
                write!(f, ".await")
            }
            FieldOperation::Nested { fields, .. } => {
                for field in fields {
                    write!(f, ".{}", field)?;
                }
                Ok(())
            }
            FieldOperation::Index { index, .. } => {
                write!(f, "[{}]", quote::quote! { #index })
            }
            FieldOperation::Chained { operations, .. } => {
                for op in operations {
                    write!(f, "{}", op)?;
                }
                Ok(())
            }
            FieldOperation::Combined {
                deref_count,
                operation,
                ..
            } => {
                for _ in 0..*deref_count {
                    write!(f, "*")?;
                }
                write!(f, "{}", operation)
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

/// Structural assertion macro for testing complex data structures.
///
/// This procedural macro generates efficient runtime assertions that check structural patterns
/// against actual values, providing detailed error messages when assertions fail. The macro
/// transforms pattern-based syntax into optimized comparison code at compile time.
///
/// See the [crate-level documentation](crate) for comprehensive guides and learning examples.
/// This documentation serves as a complete specification reference.
///
/// # Syntax Specification
///
/// ```text
/// assert_struct!(expression, TypePattern);
///
/// TypePattern ::= TypeName '{' FieldPatternList '}'
///              | '_' '{' FieldPatternList '}'  // Wildcard pattern
/// FieldPatternList ::= (FieldPattern ',')* ('..')?
/// FieldPattern ::= FieldName ':' Pattern
///              | FieldName FieldOperation ':' Pattern  
/// FieldOperation ::= ('*')+ | ('.' Identifier '(' ArgumentList? ')')
/// Pattern ::= Value | ComparisonPattern | RangePattern | RegexPattern
///          | EnumPattern | TuplePattern | SlicePattern | NestedPattern
/// ```
///
/// # Complete Pattern Reference
///
/// ## Basic Value Patterns
///
/// | Pattern | Syntax | Description | Constraints |
/// |---------|--------|-------------|-------------|
/// | **Exact Value** | `field: value` | Direct equality comparison | Must implement `PartialEq` |
/// | **String Literal** | `field: "text"` | String comparison (no `.to_string()` needed) | String or &str fields |
/// | **Explicit Equality** | `field: == value` | Same as exact value but explicit | Must implement `PartialEq` |
/// | **Inequality** | `field: != value` | Not equal comparison | Must implement `PartialEq` |
///
/// ## Comparison Patterns  
///
/// | Pattern | Syntax | Description | Constraints |
/// |---------|--------|-------------|-------------|
/// | **Greater Than** | `field: > value` | Numeric greater than | Must implement `PartialOrd` |
/// | **Greater Equal** | `field: >= value` | Numeric greater or equal | Must implement `PartialOrd` |
/// | **Less Than** | `field: < value` | Numeric less than | Must implement `PartialOrd` |
/// | **Less Equal** | `field: <= value` | Numeric less or equal | Must implement `PartialOrd` |
///
/// ## Range Patterns
///
/// | Pattern | Syntax | Description | Constraints |
/// |---------|--------|-------------|-------------|  
/// | **Inclusive Range** | `field: start..=end` | Value in inclusive range | Must implement `PartialOrd` |
/// | **Exclusive Range** | `field: start..end` | Value in exclusive range | Must implement `PartialOrd` |
/// | **Range From** | `field: start..` | Value greater or equal to start | Must implement `PartialOrd` |
/// | **Range To** | `field: ..end` | Value less than end | Must implement `PartialOrd` |
/// | **Range Full** | `field: ..` | Matches any value | No constraints |
///
/// ## String Pattern Matching
///
/// | Pattern | Syntax | Description | Constraints |
/// |---------|--------|-------------|-------------|
/// | **Regex Literal** | `field: =~ r"pattern"` | Regular expression match | Requires `regex` feature, `String`/`&str` |
/// | **Like Trait** | `field: =~ expression` | Custom pattern matching | Must implement `Like<T>` |
///
/// ## Field Operations
///
/// | Operation | Syntax | Description | Constraints |
/// |-----------|--------|-------------|-------------|
/// | **Dereference** | `*field: pattern` | Dereference smart pointer | Must implement `Deref` |
/// | **Multiple Deref** | `**field: pattern` | Multiple dereference | Must implement `Deref` (nested) |
/// | **Method Call** | `field.method(): pattern` | Call method and match result | Method must exist and return compatible type |
/// | **Method with Args** | `field.method(args): pattern` | Call method with arguments | Method must exist with compatible signature |
/// | **Tuple Method** | `(index.method(): pattern, _)` | Method on tuple element | Valid index, method exists |
///
/// ## Enum Patterns
///
/// | Pattern | Syntax | Description | Constraints |
/// |---------|--------|-------------|-------------|
/// | **Option Some** | `field: Some(pattern)` | Match Some variant with inner pattern | `Option<T>` field |
/// | **Option None** | `field: None` | Match None variant | `Option<T>` field |
/// | **Result Ok** | `field: Ok(pattern)` | Match Ok variant with inner pattern | `Result<T, E>` field |
/// | **Result Err** | `field: Err(pattern)` | Match Err variant with inner pattern | `Result<T, E>` field |
/// | **Unit Variant** | `field: EnumType::Variant` | Match unit enum variant | Enum with unit variant |
/// | **Tuple Variant** | `field: EnumType::Variant(patterns...)` | Match tuple enum variant | Enum with tuple variant |
/// | **Struct Variant** | `field: EnumType::Variant { fields... }` | Match struct enum variant | Enum with struct variant |
///
/// ## Wildcard Struct Patterns
///
/// | Pattern | Syntax | Description | Constraints |
/// |---------|--------|-------------|-------------|
/// | **Wildcard Struct** | `value: _ { fields... }` | Match struct without naming type | Must use `..` for partial matching |
/// | **Nested Wildcard** | `_ { field: _ { ... }, .. }` | Nested anonymous structs | Avoids importing nested types |
///
/// ## Collection Patterns
///
/// | Pattern | Syntax | Description | Constraints |
/// |---------|--------|-------------|-------------|
/// | **Exact Slice** | `field: [pattern, pattern, ...]` | Match exact slice elements | `Vec<T>` or slice |
/// | **Partial Head** | `field: [pattern, ..]` | Match prefix elements | `Vec<T>` or slice |
/// | **Partial Tail** | `field: [.., pattern]` | Match suffix elements | `Vec<T>` or slice |
/// | **Head and Tail** | `field: [pattern, .., pattern]` | Match first and last | `Vec<T>` or slice |
/// | **Empty Slice** | `field: []` | Match empty collection | `Vec<T>` or slice |
///
/// ## Tuple Patterns  
///
/// | Pattern | Syntax | Description | Constraints |
/// |---------|--------|-------------|-------------|
/// | **Exact Tuple** | `field: (pattern, pattern, ...)` | Match all tuple elements | Tuple type |
/// | **Wildcard Element** | `field: (pattern, _, pattern)` | Ignore specific elements | Tuple type |
/// | **Indexed Method** | `field: (0.method(): pattern, _)` | Method call on tuple element | Valid index |
///
/// # Parameters
///
/// - **`expression`**: Any expression that evaluates to a struct instance. The expression is
///   borrowed, not consumed, so the value remains available after the assertion.
/// - **`TypeName`**: The struct type name. Must exactly match the runtime type of the expression.
/// - **`{ fields }`**: Pattern specification for struct fields. Can be partial (with `..`) or exhaustive.
///
/// # Runtime Behavior
///
/// ## Evaluation Semantics
///
/// - **Non-consuming**: The macro borrows the value, leaving it available after the assertion
/// - **Expression evaluation**: The expression is evaluated exactly once before pattern matching
/// - **Short-circuit evaluation**: Patterns are evaluated left-to-right, failing fast on first mismatch  
/// - **Field order independence**: Fields can be specified in any order in the pattern
/// - **Type requirements**: All fields must have types compatible with their patterns
///
/// ## Pattern Matching Rules
///
/// ### Exhaustive vs Partial Matching
/// - **Without `..`**: All struct fields must be specified in the pattern (exhaustive)
/// - **With `..`**: Only specified fields are checked (partial matching)
/// - **Multiple `..`**: Compilation error - only one rest pattern allowed per struct
///
/// ### Field Operation Precedence
/// Field operations are applied in left-to-right order:
/// ```text
/// **field.method().other_method(): pattern
/// // Equivalent to: ((*(*field)).method()).other_method()
/// ```
///
/// ### String Literal Handling
/// - String literals (`"text"`) automatically work with `String` and `&str` fields
/// - No `.to_string()` conversion needed in patterns
/// - Comparison uses `PartialEq` implementation
///
/// # Panics
///
/// The macro panics (causing test failure) when:
///
/// ## Pattern Mismatches
/// - **Value mismatch**: Expected value doesn't equal actual value
/// - **Comparison failure**: Comparison operator condition fails (e.g., `>`, `<`)  
/// - **Range mismatch**: Value outside specified range
/// - **Enum variant mismatch**: Different enum variant than expected
/// - **Collection length mismatch**: Slice pattern length differs from actual length
/// - **None/Some mismatch**: Expected `Some` but got `None`, or vice versa
/// - **Ok/Err mismatch**: Expected `Ok` but got `Err`, or vice versa
///
/// ## Method Call Failures
/// - **Method panic**: Called method itself panics during execution
/// - **Argument evaluation panic**: Method arguments panic during evaluation
///
/// ## Regex Failures (when `regex` feature enabled)
/// - **Invalid regex**: Malformed regular expression pattern
/// - **Regex evaluation panic**: Regex engine encounters error
///
/// ## Runtime Type Issues
/// **Note**: Type mismatches are caught at compile time, not runtime.
///
/// # Compilation Errors
///
/// ## Field Validation
/// - **Nonexistent field**: Field doesn't exist on the struct type
/// - **Missing fields**: Required fields not specified (without `..`)  
/// - **Duplicate fields**: Same field specified multiple times
/// - **Invalid field operations**: Operations not supported by field type
///
/// ## Type Compatibility
/// - **Type mismatch**: Pattern type incompatible with field type
/// - **Trait requirements**: Field doesn't implement required traits (`PartialEq`, `PartialOrd`, etc.)
/// - **Method signatures**: Method doesn't exist or has incompatible signature
/// - **Deref constraints**: Field type doesn't implement `Deref` for dereference operations
///
/// ## Syntax Validation  
/// - **Invalid syntax**: Malformed pattern syntax
/// - **Invalid operators**: Unsupported operator for field type
/// - **Invalid ranges**: Malformed range expressions
/// - **Invalid regex syntax**: Invalid regex literal (when using raw strings)
/// - **Multiple rest patterns**: More than one `..` in same struct pattern
///
/// ## Feature Requirements
/// - **Missing regex feature**: Using `=~ r"pattern"` without `regex` feature enabled
/// - **Like trait not implemented**: Using `=~ expr` where `Like` trait not implemented
///
/// # Edge Cases and Limitations
///
/// ## Method Call Constraints
/// - **Return type compatibility**: Method return type must be compatible with pattern type
/// - **Argument evaluation**: Method arguments are evaluated before the method call
/// - **No generic method inference**: Generic methods may require explicit type annotations
/// - **Tuple indexing bounds**: Tuple method calls require valid index at compile time
///
/// ## Collection Pattern Limitations
/// - **Fixed length patterns**: Slice patterns without `..` require exact length match
/// - **Nested pattern complexity**: Deeply nested slice patterns may impact compile time
/// - **Memory usage**: Large literal slice patterns increase binary size
///
/// ## Smart Pointer Behavior
/// - **Multiple deref levels**: Each `*` adds one deref level, must match pointer nesting
/// - **Deref coercion**: Standard Rust deref coercion rules apply
/// - **Ownership semantics**: Dereferencing borrows the pointed-to value
///
/// ## Performance Considerations
/// - **Compile time**: Complex nested patterns increase compilation time  
/// - **Runtime overhead**: Pattern matching is zero-cost for simple patterns
/// - **Error message generation**: Error formatting only occurs on failure
///
/// # Feature Dependencies
///
/// ## Regex Feature (`regex`)
/// - **Default**: Enabled by default
/// - **Required for**: `=~ r"pattern"` syntax with string literals
/// - **Disable with**: `default-features = false` in Cargo.toml
/// - **Alternative**: Use `Like` trait with pre-compiled regex or custom patterns
///
/// ## Like Trait Extension
/// - **No feature required**: Always available
/// - **Custom implementations**: Implement `Like<T>` for custom pattern matching
/// - **Regex integration**: Built-in implementations for regex when feature enabled
///
/// # Error Message Format
///
/// When assertions fail, the macro generates structured error messages with:
///
/// ## Error Components
/// - **Error type**: Specific failure category (value mismatch, comparison failure, etc.)
/// - **Field path**: Complete path to the failing field (e.g., `response.user.profile.age`)
/// - **Source location**: File name and line number of the assertion  
/// - **Actual value**: The value that was found
/// - **Expected pattern**: The pattern that was expected to match
/// - **Pattern context**: Visual representation showing where the failure occurred
///
/// ## Error Types
/// - **value mismatch**: Direct equality comparison failed
/// - **comparison mismatch**: Comparison operator condition failed (`>`, `<`, etc.)
/// - **range mismatch**: Value outside specified range
/// - **regex mismatch**: Regex pattern didn't match
/// - **enum variant mismatch**: Wrong enum variant
/// - **slice mismatch**: Collection length or element pattern failure
/// - **method call error**: Method call or result pattern failure
///
/// ## Pattern Context Display
/// Complex patterns show visual context with failure highlighting:
/// ```text
/// assert_struct! failed:
///
///    | Response { user: User { profile: Profile {
/// comparison mismatch:
///   --> `response.user.profile.age` (tests/api.rs:45)
///    |         age: > 18,
///    |              ^^^^^ actual: 17
///    | } } }
/// ```
///
/// ## Method Call Errors
/// Method calls in field paths are clearly indicated:
/// ```text
/// comparison mismatch:
///   --> `data.items.len()` (tests/collections.rs:23)
///   actual: 3
///   expected: > 5
/// ```
///
/// # Quick Reference Examples
///
/// ```rust
/// # use assert_struct::assert_struct;
/// # #[derive(Debug)]
/// # struct Example { value: i32, name: String, items: Vec<i32> }
/// # let example = Example { value: 42, name: "test".to_string(), items: vec![1, 2] };
/// // Basic pattern matching
/// assert_struct!(example, Example {
///     value: 42,                    // Exact equality
///     name: != "other",             // Inequality  
///     items.len(): >= 2,            // Method call with comparison
///     ..                            // Partial matching
/// });
/// ```
///
/// # See Also
///
/// - **Learning Guide**: See the [crate-level documentation](crate) for comprehensive examples
/// - **Real-World Examples**: Check the `examples/` directory for practical usage patterns
/// - **Like Trait**: Implement custom pattern matching with the `Like` trait
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
