//! Simple value patterns for direct equality matching.
//!
//! Examples: 42, "hello", true, my_variable

use std::fmt;

use crate::pattern::expr_to_string;

/// Simple value pattern: 42, "hello", true
#[derive(Debug, Clone)]
pub(crate) struct PatternSimple {
    pub node_id: usize,
    pub expr: syn::Expr,
}

impl fmt::Display for PatternSimple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", expr_to_string(&self.expr))
    }
}
