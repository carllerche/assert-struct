//! Map pattern types.
//!
//! Handles map patterns: #{ "key": pattern, .. }

use std::fmt;
use syn::{Token, parse::Parse};

use crate::parse::next_node_id;
use crate::pattern::{Pattern, expr_to_string};

/// Map pattern: #{ "key": pattern, .. } for map-like structures
#[derive(Debug, Clone)]
pub(crate) struct PatternMap {
    pub node_id: usize,
    pub entries: Vec<(syn::Expr, Pattern)>,
    pub rest: bool,
}

impl fmt::Display for PatternMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{{ ")?;
        for (i, (key, value)) in self.entries.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", expr_to_string(key), value)?;
        }
        if self.rest {
            if !self.entries.is_empty() {
                write!(f, ", ")?;
            }
            write!(f, "..")?;
        }
        write!(f, " }}")
    }
}

impl Parse for PatternMap {
    /// Parses a map pattern: #{ "key": pattern, "key2": pattern, .. }
    ///
    /// # Example Input
    /// ```text
    /// #{ "name": "Alice", "age": >= 18 }
    /// #{ "key": > 5, .. }
    /// ```
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Consume the # token
        let _: Token![#] = input.parse()?;

        // Parse the braced content
        let content;
        syn::braced!(content in input);

        // Parse the map entries
        let (entries, rest) = parse_map_entries(&content)?;

        Ok(PatternMap {
            node_id: next_node_id(),
            entries,
            rest,
        })
    }
}

/// Parse map entries: comma-separated key-value pairs with optional rest pattern
/// Supports syntax like: "key1": pattern1, "key2": pattern2, ..
fn parse_map_entries(input: syn::parse::ParseStream) -> syn::Result<(Vec<(syn::Expr, Pattern)>, bool)> {
    let mut entries = Vec::new();
    let mut rest = false;

    while !input.is_empty() {
        // Check for rest pattern (..) which allows partial matching
        if input.peek(Token![..]) {
            let _: Token![..] = input.parse()?;
            rest = true;
            break;
        }

        // Parse key expression
        let key: syn::Expr = input.parse()?;

        // Expect colon separator
        let _: Token![:] = input.parse()?;

        // Parse value pattern
        let value = input.parse()?;

        entries.push((key, value));

        if input.is_empty() {
            break;
        }

        let _: Token![,] = input.parse()?;

        // Rest pattern can appear after a comma
        if input.peek(Token![..]) {
            let _: Token![..] = input.parse()?;
            rest = true;
            break;
        }
    }

    Ok((entries, rest))
}
