//! Wildcard pattern types.
//!
//! Handles _ pattern for ignoring values

use std::fmt;

/// Wildcard pattern: _ for ignoring a value while asserting it exists
#[derive(Debug, Clone)]
pub(crate) struct PatternWildcard {
    pub node_id: usize,
}

impl fmt::Display for PatternWildcard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "_")
    }
}
