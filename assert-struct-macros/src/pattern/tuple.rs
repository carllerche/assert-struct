//! Tuple pattern types and utilities.
//!
//! Handles tuple patterns including enum variants: Some(42), Event::Click(>= 0, < 100)

use std::fmt;
use syn::{Token, parse::{Parse, ParseStream}};

use crate::parse::{next_node_id, parse_pattern};
use crate::pattern::{FieldOperation, Pattern, path_to_string};

/// Tuple pattern: (10, 20) or Some(42) or None
/// Supports mixed positional and indexed elements
#[derive(Debug, Clone)]
pub(crate) struct PatternTuple {
    pub node_id: usize,
    pub path: Option<syn::Path>,
    pub elements: Vec<TupleElement>,
}

impl Parse for PatternTuple {
    /// Parses a standalone tuple pattern without a path prefix.
    ///
    /// # Example Input
    /// ```text
    /// (10, 20)
    /// (> 10, < 30)
    /// (== 5, != 10)
    /// ```
    ///
    /// This parses parenthesized tuple elements. For enum/tuple variants
    /// with a path prefix (e.g., `Some(> 30)`), the path is parsed separately
    /// in `parse_pattern` and this is called with just the parenthesized content.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);

        let elements = TupleElement::parse_comma_separated(&content)?;

        Ok(PatternTuple {
            node_id: next_node_id(),
            path: None,
            elements,
        })
    }
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

impl TupleElement {
    /// Parse a comma-separated list of tuple elements, supporting both positional and indexed syntax.
    /// Used inside tuple patterns to handle mixed syntax like ("foo", *1: "bar", "baz")
    pub(crate) fn parse_comma_separated(input: ParseStream) -> syn::Result<Vec<Self>> {
        let mut elements = Vec::new();
        let mut position = 0;

        while !input.is_empty() {
            // First, try to parse operations (like * for deref)
            let operations = FieldOperation::parse_option(input)?;

            // Check if this is an indexed element by looking for number followed by colon or method call
            let fork = input.fork();
            let is_indexed = if let Ok(_index_lit) = fork.parse::<syn::LitInt>() {
                fork.peek(Token![:]) || fork.peek(Token![.])
            } else {
                false
            };

            if is_indexed {
                // Parse indexed element: index: pattern, *index: pattern, or index.method(): pattern
                let index_lit: syn::LitInt = input.parse()?;
                let index: usize = index_lit.base10_parse()?;

                // Validate that index matches current position
                if index != position {
                    return Err(syn::Error::new_spanned(
                        index_lit,
                        format!("Index {} must match position {} in tuple", index, position),
                    ));
                }

                // Check for method calls after the index: 0.len():
                let final_operations = if input.peek(Token![.]) {
                    Some(FieldOperation::parse_chain(input, operations)?)
                } else {
                    operations
                };

                let _: Token![:] = input.parse()?;
                let pattern = parse_pattern(input)?;

                elements.push(TupleElement::Indexed {
                    index,
                    operations: final_operations,
                    pattern,
                });
            } else {
                // If we parsed operations but no index, this is an error
                if operations.is_some() {
                    return Err(syn::Error::new(
                        input.span(),
                        "Operations like * can only be used with indexed elements (e.g., *0:, *1:)",
                    ));
                }

                // Parse positional element: just a pattern
                let pattern = parse_pattern(input)?;
                elements.push(TupleElement::Positional { pattern });
            }

            position += 1;

            if !input.is_empty() {
                let _: Token![,] = input.parse()?;
            }
        }

        Ok(elements)
    }
}

impl fmt::Display for PatternTuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "{}", path_to_string(path))?;
        }
        write!(f, "(")?;
        for (i, elem) in self.elements.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", elem)?;
        }
        write!(f, ")")
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
