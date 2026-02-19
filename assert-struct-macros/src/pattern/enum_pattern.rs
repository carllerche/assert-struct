//! Enum pattern types for tuple-style enum variants.
//!
//! Handles enum tuple patterns: Some(42), Event::Click(>= 0, < 100), Ok(> 0)

use syn::parse::{Parse, ParseStream};

use crate::parse::next_node_id;
use crate::pattern::tuple::TupleElement;

/// Enum tuple pattern: Some(42), Event::Click(>= 0, < 100), or Status::Active
/// Always has a path prefix (the enum variant) and optional tuple elements
#[derive(Debug, Clone)]
pub(crate) struct PatternEnum {
    pub node_id: usize,
    pub path: syn::Path,
    pub elements: Vec<TupleElement>,
}

impl Parse for PatternEnum {
    /// Parses an enum pattern with a required path prefix.
    ///
    /// # Example Input
    /// ```text
    /// Some(> 30)
    /// Event::Click(>= 0, < 100)
    /// Ok(== 42)
    /// Status::Active    // Unit variant (no parens)
    /// ```
    ///
    /// This assumes the input starts with a path, optionally followed by parenthesized content.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let path: syn::Path = input.parse()?;

        // Check if there are parentheses (tuple variant) or not (unit variant)
        let elements = if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            TupleElement::parse_comma_separated(&content)?
        } else {
            // Unit variant - no elements
            vec![]
        };

        Ok(PatternEnum {
            node_id: next_node_id(),
            path,
            elements,
        })
    }
}
