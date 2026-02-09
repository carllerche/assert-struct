//! Slice pattern types.
//!
//! Handles slice patterns: [1, 2, 3], [> 0, < 10]

use std::fmt;

use crate::pattern::Pattern;

/// Slice pattern: [1, 2, 3] or [1, .., 5]
#[derive(Debug, Clone)]
pub(crate) struct PatternSlice {
    pub node_id: usize,
    pub elements: Vec<Pattern>,
}

impl fmt::Display for PatternSlice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, elem) in self.elements.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", elem)?;
        }
        write!(f, "]")
    }
}
