//! Wildcard pattern types.
//!
//! Handles _ pattern for ignoring values

use std::fmt;
use syn::{Token, parse::Parse};

use crate::parse::next_node_id;

/// Wildcard pattern: _ for ignoring a value while asserting it exists
#[derive(Debug, Clone)]
pub(crate) struct PatternWildcard {
    pub node_id: usize,
}

impl Parse for PatternWildcard {
    /// Parses a wildcard pattern: `_`
    ///
    /// # Example Input
    /// ```text
    /// _
    /// ```
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![_] = input.parse()?;
        Ok(PatternWildcard {
            node_id: next_node_id(),
        })
    }
}

impl fmt::Display for PatternWildcard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "_")
    }
}
