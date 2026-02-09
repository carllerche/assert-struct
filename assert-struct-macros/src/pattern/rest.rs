//! Rest pattern types.
//!
//! Handles the .. pattern for partial matching

use std::fmt;

/// Rest pattern: .. for partial matching
#[derive(Debug, Clone)]
pub(crate) struct PatternRest {
    pub node_id: usize,
}

impl fmt::Display for PatternRest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "..")
    }
}
