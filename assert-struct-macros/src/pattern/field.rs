//! Field assertion and operation types.
//!
//! These types support field access patterns in struct and tuple matching.

use std::fmt;
use syn::{Token, parse::Parse};

use crate::parse::{parse_field_operations, parse_pattern};
use crate::pattern::Pattern;

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

    /// Index operation: field[0], field[index], etc.
    /// Stores the index expression to use
    Index {
        index: syn::Expr,
        span: proc_macro2::Span,
    },

    /// Combined operation: dereferencing followed by method/nested/index access
    /// Example: *field.method(), **field.inner, *field[0], etc.
    Combined {
        deref_count: usize,
        operation: Box<FieldOperation>,
        span: proc_macro2::Span,
    },

    /// Chained operations: nested field followed by index or method
    /// Example: field.nested[0], field.inner.method(), field.sub[1].len()
    Chained {
        operations: Vec<FieldOperation>,
        span: proc_macro2::Span,
    },
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

impl Parse for FieldAssertion {
    /// Parses a single field assertion within a struct pattern.
    ///
    /// # Example Input
    /// ```text
    /// name: "Alice"
    /// age: >= 18
    /// *boxed_value: 42
    /// email: =~ r".*@example\.com"
    /// ```
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Check if we have field operations (starting with * for deref)
        let mut operations = None;
        let mut deref_count = 0;
        let span = input.span();

        // Count leading * tokens for dereferencing
        while input.peek(Token![*]) {
            let _: Token![*] = input.parse()?;
            deref_count += 1;
        }

        if deref_count > 0 {
            operations = Some(FieldOperation::Deref {
                count: deref_count,
                span,
            });
        }

        // Parse field name and potential chained operations
        let field_name: syn::Ident = input.parse()?;

        // Check for chained operations: field.method(), field.nested, field[index], etc.
        if input.peek(Token![.]) || input.peek(syn::token::Bracket) {
            operations = Some(parse_field_operations(input, operations)?);
        }

        let _: Token![:] = input.parse()?;
        let pattern = parse_pattern(input)?;

        Ok(FieldAssertion {
            field_name,
            operations,
            pattern,
        })
    }
}
