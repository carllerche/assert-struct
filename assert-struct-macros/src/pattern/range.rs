//! Range pattern types.
//!
//! Handles range patterns: 10..20, 0..=100

use syn::parse::Parse;

use crate::parse::next_node_id;

/// Range pattern: 10..20, 0..=100
#[derive(Debug, Clone)]
pub(crate) struct PatternRange {
    pub node_id: usize,
    pub expr: syn::Expr,
}

impl Parse for PatternRange {
    /// Parses a range pattern expression.
    ///
    /// # Example Input
    /// ```text
    /// 10..20
    /// 0..=100
    /// ..10
    /// 5..
    /// ```
    ///
    /// This parses any valid Rust range expression.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let expr: syn::Expr = input.parse()?;

        // Verify this is actually a range expression
        if !matches!(expr, syn::Expr::Range(_)) {
            return Err(syn::Error::new_spanned(
                &expr,
                "Expected a range expression",
            ));
        }

        Ok(PatternRange {
            node_id: next_node_id(),
            expr,
        })
    }
}
