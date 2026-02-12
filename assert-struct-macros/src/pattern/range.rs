//! Range pattern types.
//!
//! Handles range patterns: 10..20, 0..=100

use std::fmt;
use syn::parse::Parse;

use crate::parse::next_node_id;
use crate::pattern::expr_to_string;

/// Range pattern: 10..20, 0..=100
#[derive(Debug, Clone)]
pub(crate) struct PatternRange {
    pub node_id: usize,
    pub expr: syn::Expr,
}

impl PatternRange {
    /// Convert this pattern to a string for error context
    pub(crate) fn to_error_context_string(&self) -> String {
        let expr = &self.expr;
        quote::quote! { #expr }.to_string()
    }
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

impl fmt::Display for PatternRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", expr_to_string(&self.expr))
    }
}
