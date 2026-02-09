//! Regex and Like pattern types.
//!
//! Handles regex patterns (=~ r"pattern") and Like trait patterns (=~ expr)

use std::fmt;

use crate::pattern::expr_to_string;

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
impl fmt::Display for PatternRegex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, r#"=~ r"{}""#, self.pattern)
    }
}

#[cfg(feature = "regex")]
impl fmt::Display for PatternLike {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "=~ {}", expr_to_string(&self.expr))
    }
}
