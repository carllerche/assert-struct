//! Field assertion and operation types.
//!
//! These types support field access patterns in struct and tuple matching.

use std::fmt;
use syn::{Token, parse::Parse};

use crate::pattern::Pattern;

/// Represents a field name which can be either an identifier (for structs)
/// or an index (for tuples)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum FieldName {
    /// Named field: user.name, response.status
    Ident(syn::Ident),

    /// Indexed field: tuple.0, tuple.1
    Index(usize),
}

impl fmt::Display for FieldName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldName::Ident(ident) => write!(f, "{}", ident),
            FieldName::Index(index) => write!(f, "{}", index),
        }
    }
}

impl quote::ToTokens for FieldName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            FieldName::Ident(ident) => ident.to_tokens(tokens),
            FieldName::Index(index) => {
                // Convert index to a syn::Index for proper token generation
                let idx = syn::Index::from(*index);
                idx.to_tokens(tokens);
            }
        }
    }
}

impl Parse for FieldName {
    /// Parses a field name, which can be either an identifier or a numeric index.
    ///
    /// # Examples
    /// - `name` → `FieldName::Ident("name")`
    /// - `0` → `FieldName::Index(0)`
    /// - `42` → `FieldName::Index(42)`
    ///
    /// # Note on consecutive indices
    /// Due to proc macro tokenization, consecutive numeric indices like `.0.0`
    /// are tokenized as a float literal after the first dot is consumed.
    /// This is a known limitation - use tuple destructuring syntax instead.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Try to parse as a numeric literal first using fork
        let fork = input.fork();
        if let Ok(lit) = fork.parse::<syn::LitInt>() {
            // Successfully parsed as number, consume from real input
            let _: syn::LitInt = input.parse()?;
            let index = lit.base10_parse()?;
            Ok(FieldName::Index(index))
        } else {
            // Try parsing as identifier
            input
                .parse::<syn::Ident>()
                .map(FieldName::Ident)
                .map_err(|_| {
                    syn::Error::new(
                        input.span(),
                        "expected field name (identifier or numeric index)",
                    )
                })
        }
    }
}

/// Field assertion - field operations paired with an expected pattern
/// The operations represent the full path to the field (e.g., `.name`, `.0.field`, `*field.method()`)
#[derive(Debug, Clone)]
pub(crate) struct FieldAssertion {
    pub operations: FieldOperation,
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

    /// Named field access: field.name, field.inner, etc.
    /// A single step in a field chain accessing a named field
    NamedField {
        name: syn::Ident,
        span: proc_macro2::Span,
    },

    /// Unnamed field access: field.0, field.1, etc.
    /// A single step in a field chain accessing a tuple element
    UnnamedField {
        index: usize,
        span: proc_macro2::Span,
    },

    /// Index operation: field\[0\], field\[index\], etc.
    /// Stores the index expression to use
    Index {
        index: syn::Expr,
        span: proc_macro2::Span,
    },

    /// Chained operations: multiple operations in sequence
    /// Example: field.nested\[0\], field.inner.method(), *field.len(), **field.inner
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
            FieldOperation::NamedField { name, .. } => {
                write!(f, ".{}", name)
            }
            FieldOperation::UnnamedField { index, .. } => {
                write!(f, ".{}", index)
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
        let operations = input.parse()?;
        let _: Token![:] = input.parse()?;
        let pattern = input.parse()?;

        Ok(FieldAssertion {
            operations,
            pattern,
        })
    }
}

impl Parse for FieldOperation {
    /// Parse a complete field operation sequence: *field.method()[index].await
    ///
    /// # Example Input
    /// ```text
    /// name
    /// *boxed_value
    /// field.method()
    /// tuple.0.inner
    /// ```
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let span = input.span();
        let mut operations = Vec::new();

        // Parse leading derefs
        let mut deref_count = 0;
        while input.peek(Token![*]) {
            let _: Token![*] = input.parse()?;
            deref_count += 1;
        }
        if deref_count > 0 {
            operations.push(FieldOperation::Deref {
                count: deref_count,
                span,
            });
        }

        // Parse field name (required)
        let field_name: FieldName = input.parse()?;
        let field_op = match field_name {
            FieldName::Ident(ident) => FieldOperation::NamedField { name: ident, span },
            FieldName::Index(index) => FieldOperation::UnnamedField { index, span },
        };
        operations.push(field_op);

        // Parse additional operations (.field, .method(), [index], .await)
        while input.peek(Token![.]) || input.peek(syn::token::Bracket) {
            FieldOperation::parse_one_into(input, &mut operations)?;
        }

        // Convert Vec to single operation or Chained
        let final_operation = match operations.len() {
            0 => unreachable!("Must have at least field name"),
            1 => operations.into_iter().next().unwrap(),
            _ => FieldOperation::Chained { operations, span },
        };

        Ok(final_operation)
    }
}

impl FieldOperation {
    /// Get the root field name from this operation
    /// For NamedField/UnnamedField, returns that name
    /// For Chained, recursively finds the first field access (skipping Deref operations)
    pub(crate) fn root_field_name(&self) -> FieldName {
        match self {
            FieldOperation::NamedField { name, .. } => FieldName::Ident(name.clone()),
            FieldOperation::UnnamedField { index, .. } => FieldName::Index(*index),
            FieldOperation::Chained { operations, .. } => {
                // Find the first non-Deref operation and get its root field name
                operations
                    .iter()
                    .find(|op| !matches!(op, FieldOperation::Deref { .. }))
                    .expect("Chained operation must have at least one non-Deref operation")
                    .root_field_name()
            }
            _ => panic!("Cannot extract root field name from {:?}", self),
        }
    }

    /// Get operations after the root field access (tail operations)
    /// For NamedField/UnnamedField alone, returns None (no additional operations)
    /// For Chained, returns all operations except the first non-Deref field access
    ///
    /// Examples:
    /// - Chained([Deref, NamedField("x")]) → Some(Deref)
    /// - Chained([NamedField("x"), Method("len")]) → Some(Method("len"))
    /// - Chained([Deref, NamedField("x"), Method("len")]) → Some(Chained([Deref, Method("len")]))
    pub(crate) fn tail_operations(&self) -> Option<Self> {
        match self {
            FieldOperation::NamedField { .. } | FieldOperation::UnnamedField { .. } => {
                // Just a field access, no additional operations
                None
            }
            FieldOperation::Chained { operations, span } => {
                // Find the index of the first non-Deref field access
                let field_access_idx = operations
                    .iter()
                    .position(|op| {
                        matches!(
                            op,
                            FieldOperation::NamedField { .. } | FieldOperation::UnnamedField { .. }
                        )
                    })
                    .expect("Chained operation must have at least one field access");

                // Collect all operations except the field access itself
                let mut tail_ops: Vec<_> = operations[..field_access_idx].to_vec();
                tail_ops.extend_from_slice(&operations[field_access_idx + 1..]);

                if tail_ops.is_empty() {
                    None
                } else if tail_ops.len() == 1 {
                    Some(tail_ops.into_iter().next().unwrap())
                } else {
                    Some(FieldOperation::Chained {
                        operations: tail_ops,
                        span: *span,
                    })
                }
            }
            // For other operation types (Method, Await, Index, Deref), they don't have a root field to strip
            // These should not appear at the root level of a FieldAssertion, but if they do,
            // return None to indicate no tail
            _ => None,
        }
    }
}

impl FieldOperation {
    /// Parse a dot operation: .await, .field, .method(), or .0
    /// Pushes the parsed operation(s) into the provided Vec
    fn parse_one_dot_into(
        input: syn::parse::ParseStream,
        ops: &mut Vec<FieldOperation>,
    ) -> syn::Result<()> {
        let dot_span = input.span();
        let _: Token![.] = input.parse()?;

        if input.peek(Token![await]) {
            let await_span = input.span();
            let _: Token![await] = input.parse()?;
            ops.push(FieldOperation::Await { span: await_span });
            Ok(())
        } else if input.peek(syn::LitInt) {
            // It's a tuple index like .0 or .1
            let lit_int: syn::LitInt = input.parse()?;
            let index: usize = lit_int.base10_parse()?;
            ops.push(FieldOperation::UnnamedField {
                index,
                span: dot_span,
            });
            Ok(())
        } else if input.peek(syn::LitFloat) {
            let lit_float: syn::LitFloat = input.parse()?;
            // Parse float like "0.0" and split into two UnnamedField operations
            let float_str = lit_float.to_string();
            let Some((first, second)) = float_str.split_once('.') else {
                return Err(syn::Error::new(
                    dot_span,
                    "Invalid float literal in field access",
                ));
            };

            let first_idx = first
                .parse::<usize>()
                .map_err(|_| syn::Error::new(dot_span, "Invalid numeric index in field access"))?;
            let second_idx = second
                .parse::<usize>()
                .map_err(|_| syn::Error::new(dot_span, "Invalid numeric index in field access"))?;

            // Push two sequential UnnamedField operations
            ops.push(FieldOperation::UnnamedField {
                index: first_idx,
                span: dot_span,
            });
            ops.push(FieldOperation::UnnamedField {
                index: second_idx,
                span: dot_span,
            });
            Ok(())
        } else {
            // Parse as identifier for named field
            let ident: syn::Ident = input.parse()?;

            // Check if this is a method call
            if input.peek(syn::token::Paren) {
                let args_content;
                syn::parenthesized!(args_content in input);

                let mut args = Vec::new();
                while !args_content.is_empty() {
                    let arg: syn::Expr = args_content.parse()?;
                    args.push(arg);

                    if !args_content.peek(Token![,]) {
                        break;
                    }
                    let _: Token![,] = args_content.parse()?;
                }

                ops.push(FieldOperation::Method {
                    name: ident,
                    args,
                    span: dot_span,
                });
                Ok(())
            } else {
                // Single named field access
                ops.push(FieldOperation::NamedField {
                    name: ident,
                    span: dot_span,
                });
                Ok(())
            }
        }
    }

    /// Parse a single operation: .await, .field, .method(), or \[index\]
    /// Pushes the parsed operation into the provided Vec
    pub(crate) fn parse_one_into(
        input: syn::parse::ParseStream,
        ops: &mut Vec<FieldOperation>,
    ) -> syn::Result<()> {
        if input.peek(Token![.]) {
            Self::parse_one_dot_into(input, ops)
        } else if input.peek(syn::token::Bracket) {
            // Index operation - need to capture the span that encompasses the bracket
            let content;
            let bracket_token = syn::bracketed!(content in input);
            let index: syn::Expr = content.parse()?;
            ops.push(FieldOperation::Index {
                index,
                span: bracket_token.span.open(),
            });
            Ok(())
        } else {
            Err(syn::Error::new(
                input.span(),
                "Expected field operation (.field, .method(), .await, or [index])",
            ))
        }
    }
}
