//! Regex and Like pattern types.
//!
//! Handles regex patterns (=~ r"pattern") and Like trait patterns (=~ expr)

use syn::{Token, parse::Parse};

use crate::parse::next_node_id;

/// Regex pattern: =~ "pattern" - string literal optimized at compile time
#[cfg(feature = "regex")]
#[derive(Debug, Clone)]
pub(crate) struct PatternRegex {
    pub node_id: usize,
    pub pattern: String,
    pub span: proc_macro2::Span,
}

/// Like pattern: =~ expr - arbitrary expression using Like trait
#[cfg(feature = "regex")]
#[derive(Debug, Clone)]
pub(crate) struct PatternLike {
    pub node_id: usize,
    pub expr: syn::Expr,
}

#[cfg(feature = "regex")]
impl PatternLike {
    /// Convert this PatternLike into a Pattern, optimizing string literals to PatternRegex.
    ///
    /// If the expression is a string literal, it will be converted to PatternRegex for
    /// compile-time regex compilation. Otherwise, it returns Pattern::Like for runtime
    /// pattern matching using the Like trait.
    pub(crate) fn into_pattern(self) -> crate::pattern::Pattern {
        if let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) = &self.expr
        {
            // String literal - compile regex at macro expansion time
            crate::pattern::Pattern::Regex(PatternRegex {
                node_id: self.node_id,
                pattern: lit_str.value(),
                span: lit_str.span(),
            })
        } else {
            // Expression - use Like trait at runtime
            crate::pattern::Pattern::Like(self)
        }
    }
}

#[cfg(feature = "regex")]
impl Parse for PatternLike {
    /// Parses a Like pattern: `=~ expr`
    ///
    /// # Example Input
    /// ```text
    /// =~ r"pattern"
    /// =~ my_pattern
    /// =~ get_pattern()
    /// ```
    ///
    /// This parses the `=~` operator followed by any expression.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _: Token![=] = input.parse()?;
        let _: Token![~] = input.parse()?;
        let expr = input.parse::<syn::Expr>()?;

        Ok(PatternLike {
            node_id: next_node_id(),
            expr,
        })
    }
}
