//! Map pattern types.
//!
//! Handles map patterns: #{ "key": pattern, .. }

use std::fmt;

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
