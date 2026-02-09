//! Closure pattern types.
//!
//! Handles closure patterns: |x| x > 5

use std::fmt;

/// Closure pattern: |x| expr for custom validation (escape hatch)
#[derive(Debug, Clone)]
pub(crate) struct PatternClosure {
    pub node_id: usize,
    pub closure: syn::ExprClosure,
}

impl fmt::Display for PatternClosure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let closure = &self.closure;
        write!(f, "{}", quote::quote! { #closure })
    }
}
