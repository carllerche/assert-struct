//! Simple value patterns for direct equality matching.
//!
//! Examples: 42, "hello", true, my_variable

use syn::parse::Parse;

use crate::parse::next_node_id;

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
