//! Rest pattern types.
//!
//! Handles the .. pattern for partial matching

use std::fmt;
use syn::{Token, parse::Parse};

use crate::parse::next_node_id;

/// Rest pattern: .. for partial matching
#[derive(Debug, Clone)]
pub(crate) struct PatternRest {
    pub node_id: usize,
}

impl Parse for PatternRest {
    /// Parses a rest pattern: `..`
    ///
    /// # Example Input
    /// ```text
    /// ..
    /// ```
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![..] = input.parse()?;
        Ok(PatternRest {
            node_id: next_node_id(),
        })
    }
}

impl fmt::Display for PatternRest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "..")
    }
}
