//! Pattern types for structural assertions.
//!
//! This module defines the various pattern types that can be used in assertions,
//! along with helper types for field operations and tuple elements.

mod comparison;

pub(crate) use comparison::{ComparisonOp, PatternComparison};

use std::fmt;
use syn::{Token, punctuated::Punctuated};

/// Unified pattern type that can represent any pattern
#[derive(Debug, Clone)]
pub(crate) enum Pattern {
    Simple(PatternSimple),
    Struct(PatternStruct),
    Tuple(PatternTuple),
    Slice(PatternSlice),
    Comparison(PatternComparison),
    Range(PatternRange),
    #[cfg(feature = "regex")]
    Regex(PatternRegex),
    #[cfg(feature = "regex")]
    Like(PatternLike),
    Rest(PatternRest),
    Wildcard(PatternWildcard),
    Closure(PatternClosure),
    Map(PatternMap),
}

/// Simple value pattern: 42, "hello", true
#[derive(Debug, Clone)]
pub(crate) struct PatternSimple {
    pub node_id: usize,
    pub expr: syn::Expr,
}

/// Struct pattern: User { name: "Alice", age: 30, .. }
/// When path is None, it's a wildcard pattern: _ { name: "Alice", .. }
#[derive(Debug, Clone)]
pub(crate) struct PatternStruct {
    pub node_id: usize,
    pub path: Option<syn::Path>,
    pub fields: Punctuated<FieldAssertion, Token![,]>,
    pub rest: bool,
}

/// Tuple pattern: (10, 20) or Some(42) or None
/// Supports mixed positional and indexed elements
#[derive(Debug, Clone)]
pub(crate) struct PatternTuple {
    pub node_id: usize,
    pub path: Option<syn::Path>,
    pub elements: Vec<TupleElement>,
}

/// Slice pattern: [1, 2, 3] or [1, .., 5]
#[derive(Debug, Clone)]
pub(crate) struct PatternSlice {
    pub node_id: usize,
    pub elements: Vec<Pattern>,
}

/// Range pattern: 10..20, 0..=100
#[derive(Debug, Clone)]
pub(crate) struct PatternRange {
    pub node_id: usize,
    pub expr: syn::Expr,
}

/// Regex pattern: =~ "pattern" - string literal optimized at compile time
#[cfg(feature = "regex")]
#[derive(Debug, Clone)]
pub(crate) struct PatternRegex {
    pub node_id: usize,
    pub pattern: String,
    pub span: proc_macro2::Span,
}

/// Like pattern: =~ expr - arbitrary expression using Like trait
#[cfg(feature = "regex")]
#[derive(Debug, Clone)]
pub(crate) struct PatternLike {
    pub node_id: usize,
    pub expr: syn::Expr,
}

/// Rest pattern: .. for partial matching
#[derive(Debug, Clone)]
pub(crate) struct PatternRest {
    pub node_id: usize,
}

/// Wildcard pattern: _ for ignoring a value while asserting it exists
#[derive(Debug, Clone)]
pub(crate) struct PatternWildcard {
    pub node_id: usize,
}

/// Closure pattern: |x| expr for custom validation (escape hatch)
#[derive(Debug, Clone)]
pub(crate) struct PatternClosure {
    pub node_id: usize,
    pub closure: syn::ExprClosure,
}

/// Map pattern: #{ "key": pattern, .. } for map-like structures
#[derive(Debug, Clone)]
pub(crate) struct PatternMap {
    pub node_id: usize,
    pub entries: Vec<(syn::Expr, Pattern)>,
    pub rest: bool,
}

/// Field assertion - a field name paired with its expected pattern
/// Supports operations like dereferencing, method calls, and nested access
#[derive(Debug, Clone)]
pub(crate) struct FieldAssertion {
    pub field_name: syn::Ident,
    pub operations: Option<FieldOperation>,
    pub pattern: Pattern,
}

/// Represents an operation to be performed on a field before pattern matching
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) enum FieldOperation {
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

/// Represents an element in a tuple pattern, supporting both positional and indexed syntax
#[derive(Debug, Clone)]
pub(crate) enum TupleElement {
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

// Helper function to format syn expressions as strings
fn expr_to_string(expr: &syn::Expr) -> String {
    // This is a simplified version - in production we'd want more complete handling
    match expr {
        syn::Expr::Lit(lit) => {
            // Handle literals
            quote::quote! { #lit }.to_string()
        }
        syn::Expr::Path(path) => {
            // Handle paths
            quote::quote! { #path }.to_string()
        }
        syn::Expr::Range(range) => {
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
            Pattern::Simple(p) => write!(f, "{}", expr_to_string(&p.expr)),
            Pattern::Struct(p) => {
                if let Some(path) = &p.path {
                    write!(f, "{} {{ ", path_to_string(path))?;
                } else {
                    write!(f, "_ {{ ")?;
                }
                for (i, field) in p.fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field.field_name, field.pattern)?;
                }
                if p.rest {
                    if !p.fields.is_empty() {
                        write!(f, ", ")?;
                    }
                    write!(f, "..")?;
                }
                write!(f, " }}")
            }
            Pattern::Tuple(p) => {
                if let Some(path) = &p.path {
                    write!(f, "{}", path_to_string(path))?;
                }
                write!(f, "(")?;
                for (i, elem) in p.elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, ")")
            }
            Pattern::Slice(p) => {
                write!(f, "[")?;
                for (i, elem) in p.elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
            Pattern::Comparison(p) => {
                write!(f, "{} {}", p.op, expr_to_string(&p.expr))
            }
            Pattern::Range(p) => {
                write!(f, "{}", expr_to_string(&p.expr))
            }
            #[cfg(feature = "regex")]
            Pattern::Regex(p) => {
                write!(f, r#"=~ r"{}""#, p.pattern)
            }
            #[cfg(feature = "regex")]
            Pattern::Like(p) => {
                write!(f, "=~ {}", expr_to_string(&p.expr))
            }
            Pattern::Rest(_) => {
                write!(f, "..")
            }
            Pattern::Wildcard(_) => {
                write!(f, "_")
            }
            Pattern::Closure(p) => {
                let closure = &p.closure;
                write!(f, "{}", quote::quote! { #closure })
            }
            Pattern::Map(p) => {
                write!(f, "#{{ ")?;
                for (i, (key, value)) in p.entries.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", expr_to_string(key), value)?;
                }
                if p.rest {
                    if !p.entries.is_empty() {
                        write!(f, ", ")?;
                    }
                    write!(f, "..")?;
                }
                write!(f, " }}")
            }
        }
    }
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
