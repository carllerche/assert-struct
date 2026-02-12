//! String literal patterns for direct equality matching.
//!
//! Examples: "hello", "world"

use std::fmt;
use syn::LitStr;

use crate::parse::next_node_id;

/// String literal pattern: "hello", "world"
/// Separated from PatternSimple to handle the special .as_ref() logic during expansion
#[derive(Debug, Clone)]
pub(crate) struct PatternString {
    pub node_id: usize,
    pub lit: LitStr,
}

impl PatternString {
    pub fn new(lit: LitStr) -> Self {
        PatternString {
            node_id: next_node_id(),
            lit,
        }
    }

    /// Convert this pattern to a string for error context
    pub(crate) fn to_error_context_string(&self) -> String {
        format!("\"{}\"", self.lit.value())
    }
}

impl fmt::Display for PatternString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.lit.value())
    }
}
