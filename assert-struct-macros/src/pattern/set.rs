//! Set pattern types.
//!
//! Handles set patterns: #(pattern, pattern, ..)

use syn::{Token, parse::Parse};

use crate::parse::next_node_id;
use crate::pattern::Pattern;

/// Set pattern: #(pattern, ..) for unordered collection matching.
///
/// Each element pattern must match a distinct element of the collection,
/// in any order. Matching uses backtracking to find a valid assignment.
#[derive(Debug, Clone)]
pub(crate) struct PatternSet {
    pub node_id: usize,
    pub span: proc_macro2::Span,
    pub elements: Vec<Pattern>,
    pub rest: bool,
}

impl Parse for PatternSet {
    /// Parses a set pattern: #(pattern, pattern, ..)
    ///
    /// # Example Input
    /// ```text
    /// #(1, 2, 3)
    /// #(> 0, < 10, ..)
    /// #(_ { kind: "click", .. }, ..)
    /// ```
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Consume the # token
        let _: Token![#] = input.parse()?;

        // Capture the span of the `(` token before consuming it
        let span = input.span();
        let content;
        syn::parenthesized!(content in input);

        let mut elements = Vec::new();
        let mut rest = false;

        while !content.is_empty() {
            // Check for rest pattern (..)
            if content.peek(Token![..]) {
                let _: Token![..] = content.parse()?;
                rest = true;
                // Optional trailing comma
                if content.peek(Token![,]) {
                    let _: Token![,] = content.parse()?;
                }
                break;
            }

            elements.push(content.parse()?);

            if content.is_empty() {
                break;
            }

            let _: Token![,] = content.parse()?;

            // Rest pattern can appear after a comma
            if content.peek(Token![..]) {
                let _: Token![..] = content.parse()?;
                rest = true;
                break;
            }
        }

        Ok(PatternSet {
            node_id: next_node_id(),
            span,
            elements,
            rest,
        })
    }
}
