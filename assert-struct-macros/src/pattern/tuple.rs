//! Tuple pattern types and utilities.
//!
//! Handles raw tuple patterns: (10, 20), (> 10, < 30)

use std::fmt;
use syn::{
    Token,
    parse::{Parse, ParseStream},
};

use crate::parse::next_node_id;
use crate::pattern::field::FieldName;
use crate::pattern::{FieldAssertion, FieldOperation, Pattern};

/// Tuple pattern: (10, 20) or (> 10, < 30)
/// Supports mixed positional and indexed elements
/// Note: Enum tuple variants like Some(42) are handled by PatternEnum
#[derive(Debug, Clone)]
pub(crate) struct PatternTuple {
    pub node_id: usize,
    pub span: proc_macro2::Span,
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
    /// with a path prefix (e.g., `Some(> 30)`), use PatternEnum instead.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Capture the span of the `(` token before consuming it.
        let span = input.span();
        let content;
        syn::parenthesized!(content in input);

        let elements = TupleElement::parse_comma_separated(&content)?;

        Ok(PatternTuple {
            node_id: next_node_id(),
            span,
            elements,
        })
    }
}

/// Represents an element in a tuple pattern, supporting both positional and indexed syntax
///
/// Indexed elements are essentially field assertions where the field name is a numeric
/// identifier (e.g., "0", "1", "2"). This unifies tuple indexing with struct field access,
/// allowing all field operations to work seamlessly.
#[derive(Debug, Clone)]
pub(crate) enum TupleElement {
    /// Positional element in sequence order
    /// Example: "foo", > 10, Some(42)
    /// Boxed to reduce enum size since Pattern is large
    Positional(Box<Pattern>),

    /// Indexed element with explicit numeric field name
    /// The field_name will be a synthetic ident like "_0", "_1", "_2"
    /// Boxed to reduce enum size due to larger FieldAssertion struct
    Indexed(Box<FieldAssertion>),
}

impl TupleElement {
    /// Parse a comma-separated list of tuple elements, supporting both positional and indexed syntax.
    /// Used inside tuple patterns to handle mixed syntax like ("foo", *1: "bar", "baz")
    pub(crate) fn parse_comma_separated(input: ParseStream) -> syn::Result<Vec<Self>> {
        let mut elements = Vec::new();
        let mut position = 0;

        while !input.is_empty() {
            // Try to parse as indexed element by attempting FieldOperation parse
            let fork = input.fork();

            if fork.parse::<Pattern>().is_ok() && !fork.peek(Token![:]) {
                // Parse as positional pattern
                let pattern = input.parse()?;
                elements.push(TupleElement::Positional(Box::new(pattern)));
            } else {
                // Parse as indexed element
                let operations: FieldOperation = input.parse()?;
                let root_field = operations.root_field_name();

                // Validate that the index matches the current position
                match root_field {
                    FieldName::Index(index) if index == position => {
                        // Valid indexed element
                        let _: Token![:] = input.parse()?;
                        let pattern = input.parse()?;

                        elements.push(TupleElement::Indexed(Box::new(FieldAssertion {
                            operations,
                            pattern,
                        })));
                    }
                    FieldName::Index(index) => {
                        // Index doesn't match position
                        return Err(syn::Error::new(
                            input.span(),
                            format!("Index {} must match position {} in tuple", index, position),
                        ));
                    }
                    FieldName::Ident(_) => {
                        // Index doesn't match position
                        return Err(syn::Error::new(
                            input.span(),
                            "Operations like * can only be used with indexed elements (e.g., *0:, *1:)",
                        ));
                    }
                }
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
            TupleElement::Positional(pattern) => {
                write!(f, "{}", pattern)
            }
            TupleElement::Indexed(boxed_field_assertion) => {
                let field_assertion = boxed_field_assertion.as_ref();
                let operations = &field_assertion.operations;
                let pattern = &field_assertion.pattern;

                // For indexed tuple elements, just display the operations followed by the pattern
                // The operations will include the index (as UnnamedField or in a chain)
                write!(f, "{}: {}", operations, pattern)
            }
        }
    }
}
