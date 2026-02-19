//! String literal patterns for direct equality matching.
//!
//! Examples: "hello", "world"

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
}
