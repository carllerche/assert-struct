//! Simple value patterns for direct equality matching.
//!
//! Examples: 42, "hello", true, my_variable

use std::fmt;
use syn::parse::Parse;

use crate::parse::next_node_id;
use crate::pattern::expr_to_string;

/// Simple value pattern: 42, "hello", true
#[derive(Debug, Clone)]
pub(crate) struct PatternSimple {
    pub node_id: usize,
    pub expr: syn::Expr,
}

impl Parse for PatternSimple {
    /// Parses a simple expression pattern.
    ///
    /// # Example Input
    /// ```text
    /// 42
    /// "hello"
    /// true
    /// my_variable
    /// compute_value()
    /// ```
    ///
    /// This parses any valid Rust expression except ranges (handled by PatternRange).
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expr = input.parse::<syn::Expr>()?;

        Ok(PatternSimple {
            node_id: next_node_id(),
            expr,
        })
    }
}

impl fmt::Display for PatternSimple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", expr_to_string(&self.expr))
    }
}
