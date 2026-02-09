//! Range pattern types.
//!
//! Handles range patterns: 10..20, 0..=100

use std::fmt;

use crate::pattern::expr_to_string;

/// Range pattern: 10..20, 0..=100
#[derive(Debug, Clone)]
pub(crate) struct PatternRange {
    pub node_id: usize,
    pub expr: syn::Expr,
}

impl fmt::Display for PatternRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", expr_to_string(&self.expr))
    }
}
