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
            input.parse::<syn::Ident>().map(FieldName::Ident)
                .map_err(|_| syn::Error::new(
                    input.span(),
                    "expected field name (identifier or numeric index)"
                ))
        }
    }
}

/// Field assertion - a field name paired with its expected pattern
/// Supports operations like dereferencing, method calls, and nested access
#[derive(Debug, Clone)]
pub(crate) struct FieldAssertion {
    pub field_name: FieldName,
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

        // Parse field name (can be identifier or numeric index)
        let field_name: FieldName = input.parse()?;

        // Check for chained operations: field.method(), field.nested, field[index], etc.
        if input.peek(Token![.]) || input.peek(syn::token::Bracket) {
            operations = Some(parse_field_operations(input, operations)?);
        }

        let _: Token![:] = input.parse()?;
        let pattern = input.parse()?;

        Ok(FieldAssertion {
            field_name,
            operations,
            pattern,
        })
    }
}

/// Parse field operations starting from the first field name
/// Handles chained operations like .field, \[index\], .method(), .await, etc.
pub(crate) fn parse_field_operations(
    input: syn::parse::ParseStream,
    existing_operations: Option<FieldOperation>,
) -> syn::Result<FieldOperation> {
    let span = input.span();
    let mut operations = vec![];

    // Continue parsing operations in the chain using the helper
    while input.peek(Token![.]) || input.peek(syn::token::Bracket) {
        operations.push(input.parse()?);
    }

    // Build the final operation
    let final_operation = if operations.len() == 1 {
        operations.into_iter().next().unwrap()
    } else if operations.is_empty() {
        return Err(syn::Error::new(input.span(), "Expected field operations"));
    } else {
        FieldOperation::Chained { operations, span }
    };

    // Combine with existing operations if present
    if let Some(FieldOperation::Deref {
        count,
        span: deref_span,
    }) = existing_operations
    {
        Ok(FieldOperation::Combined {
            deref_count: count,
            operation: Box::new(final_operation),
            span: deref_span,
        })
    } else {
        Ok(final_operation)
    }
}

impl FieldOperation {
    /// Parse optional operations for tuple elements (currently just dereferencing)
    /// This is simpler than field operations since we only support * for now
    /// Returns None if no operations are present
    pub(crate) fn parse_option(input: syn::parse::ParseStream) -> syn::Result<Option<Self>> {
        let mut deref_count = 0;
        let span = input.span();

        // Count leading * tokens for dereferencing
        while input.peek(Token![*]) {
            let _: Token![*] = input.parse()?;
            deref_count += 1;
        }

        if deref_count > 0 {
            Ok(Some(FieldOperation::Deref {
                count: deref_count,
                span,
            }))
        } else {
            Ok(None)
        }
    }

    /// Parse a chain of operations: .method().await\[0\].field, etc.
    /// Returns a FieldOperation with appropriate chaining
    pub(crate) fn parse_chain(
        input: syn::parse::ParseStream,
        existing_operations: Option<Self>,
    ) -> syn::Result<Self> {
        let span = input.span();
        let mut operations = vec![];

        // Parse the first operation (which should start with . or [)
        operations.push(input.parse()?);

        // Continue parsing while we see . or [
        while input.peek(Token![.]) || input.peek(syn::token::Bracket) {
            operations.push(input.parse()?);
        }

        // Build the final operation
        let final_operation = if operations.len() == 1 {
            operations.into_iter().next().unwrap()
        } else {
            FieldOperation::Chained { operations, span }
        };

        // Combine with existing operations if present
        if let Some(FieldOperation::Deref {
            count,
            span: deref_span,
        }) = existing_operations
        {
            Ok(FieldOperation::Combined {
                deref_count: count,
                operation: Box::new(final_operation),
                span: deref_span,
            })
        } else {
            Ok(final_operation)
        }
    }
}

impl Parse for FieldOperation {
    /// Parse a single operation: .await, .field, .method(), or \[index\]
    /// This parses exactly one operation and returns it
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![.]) {
            let dot_span = input.span();
            let _: Token![.] = input.parse()?;

            if input.peek(Token![await]) {
                let await_span = input.span();
                let _: Token![await] = input.parse()?;
                Ok(FieldOperation::Await { span: await_span })
            } else {
                // Try to parse as integer first (for tuple indices)
                let fork = input.fork();
                if let Ok(lit_int) = fork.parse::<syn::LitInt>() {
                    // It's a tuple index like .0 or .1
                    let _: syn::LitInt = input.parse()?;
                    let index: usize = lit_int.base10_parse()?;
                    return Ok(FieldOperation::UnnamedField {
                        index,
                        span: dot_span,
                    });
                }

                // Try to parse as float (handles .0.0 tokenized as 0.0)
                let fork = input.fork();
                if let Ok(lit_float) = fork.parse::<syn::LitFloat>() {
                    // Parse float like "0.0" and split into two UnnamedField operations
                    let float_str = lit_float.to_string();
                    if let Some((first, second)) = float_str.split_once('.') {
                        if let (Ok(first_idx), Ok(second_idx)) = (first.parse::<usize>(), second.parse::<usize>()) {
                            // Consume the float from real input
                            let _: syn::LitFloat = input.parse()?;

                            // Create two chained UnnamedField operations
                            return Ok(FieldOperation::Chained {
                                operations: vec![
                                    FieldOperation::UnnamedField { index: first_idx, span: dot_span },
                                    FieldOperation::UnnamedField { index: second_idx, span: dot_span },
                                ],
                                span: dot_span,
                            });
                        }
                    }
                }

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

                    Ok(FieldOperation::Method {
                        name: ident,
                        args,
                        span: dot_span,
                    })
                } else {
                    // Single named field access
                    Ok(FieldOperation::NamedField {
                        name: ident,
                        span: dot_span,
                    })
                }
            }
        } else if input.peek(syn::token::Bracket) {
            // Index operation - need to capture the span that encompasses the bracket
            let content;
            let bracket_token = syn::bracketed!(content in input);
            let index: syn::Expr = content.parse()?;
            Ok(FieldOperation::Index {
                index,
                span: bracket_token.span.open(),
            })
        } else {
            Err(syn::Error::new(
                input.span(),
                "Expected field operation (.field, .method(), .await, or [index])",
            ))
        }
    }
}
