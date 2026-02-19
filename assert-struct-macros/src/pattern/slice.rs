//! Slice pattern types.
//!
//! Handles slice patterns: [1, 2, 3], [> 0, < 10]

use syn::{Token, parse::Parse};

use crate::parse::next_node_id;
use crate::pattern::Pattern;

/// Slice pattern: [1, 2, 3] or [1, .., 5]
#[derive(Debug, Clone)]
pub(crate) struct PatternSlice {
    pub node_id: usize,
    pub span: proc_macro2::Span,
    pub elements: Vec<Pattern>,
}

impl Parse for PatternSlice {
    /// Parses a slice pattern: [pattern, pattern, ...]
    ///
    /// # Example Input
    /// ```text
    /// [1, 2, 3]
    /// [> 0, < 10, == 5]
    /// ```
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Capture the span of the `[` token before consuming it.
        let span = input.span();
        let content;
        syn::bracketed!(content in input);

        // Parse the comma-separated list of patterns
        let elements = parse_pattern_list(&content)?;

        Ok(PatternSlice {
            node_id: next_node_id(),
            span,
            elements,
        })
    }
}

/// Parse a comma-separated list of patterns inside brackets.
fn parse_pattern_list(input: syn::parse::ParseStream) -> syn::Result<Vec<Pattern>> {
    let mut patterns = Vec::new();

    while !input.is_empty() {
        patterns.push(input.parse()?);

        if !input.is_empty() {
            let _: Token![,] = input.parse()?;
        }
    }

    Ok(patterns)
}
