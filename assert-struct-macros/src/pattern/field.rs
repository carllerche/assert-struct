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
        let ident: syn::Ident = input.parse()?;
        let field_name = FieldName::Ident(ident);

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
                let ident: syn::Ident = input.parse()?;
                let method_span = ident.span();

                if input.peek(syn::token::Paren) {
                    // Method call with args
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
                        span: method_span,
                    })
                } else {
                    // Field access - might be chained like .field.nested.deep
                    let mut fields = vec![ident];

                    // Continue parsing consecutive field accesses
                    while input.peek(Token![.])
                        && !input.peek2(Token![await])
                        && !input.peek2(syn::token::Paren)
                        && !input.peek2(syn::token::Bracket)
                    {
                        let _: Token![.] = input.parse()?;
                        let field: syn::Ident = input.parse()?;
                        fields.push(field);
                    }

                    Ok(FieldOperation::Nested {
                        fields,
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
