//! Pattern types for structural assertions.
//!
//! This module defines the various pattern types that can be used in assertions,
//! along with helper types for field operations and tuple elements.

mod closure;
mod comparison;
mod field;
mod map;
mod range;
mod rest;
mod simple;
mod slice;
mod struct_pattern;
mod tuple;
mod wildcard;

#[cfg(feature = "regex")]
mod regex;

// Re-export all pattern types
pub(crate) use closure::PatternClosure;
pub(crate) use comparison::{ComparisonOp, PatternComparison};
pub(crate) use field::{FieldAssertion, FieldOperation};
pub(crate) use map::PatternMap;
pub(crate) use range::PatternRange;
pub(crate) use rest::PatternRest;
pub(crate) use simple::PatternSimple;
pub(crate) use slice::PatternSlice;
pub(crate) use struct_pattern::PatternStruct;
pub(crate) use tuple::{PatternTuple, TupleElement};
pub(crate) use wildcard::PatternWildcard;

#[cfg(feature = "regex")]
pub(crate) use regex::{PatternLike, PatternRegex};

use std::fmt;
use syn::{Token, parse::ParseStream};

/// Unified pattern type that can represent any pattern
#[derive(Debug, Clone)]
pub(crate) enum Pattern {
    Simple(PatternSimple),
    Struct(PatternStruct),
    Tuple(PatternTuple),
    Slice(PatternSlice),
    Comparison(PatternComparison),
    Range(PatternRange),
    #[cfg(feature = "regex")]
    Regex(PatternRegex),
    #[cfg(feature = "regex")]
    Like(PatternLike),
    Rest(PatternRest),
    Wildcard(PatternWildcard),
    Closure(PatternClosure),
    Map(PatternMap),
}

// Helper functions that are used across patterns
pub(crate) fn expr_to_string(expr: &syn::Expr) -> String {
    // This is a simplified version - in production we'd want more complete handling
    match expr {
        syn::Expr::Lit(lit) => {
            // Handle literals
            quote::quote! { #lit }.to_string()
        }
        syn::Expr::Path(path) => {
            // Handle paths
            quote::quote! { #path }.to_string()
        }
        syn::Expr::Range(range) => {
            // Handle ranges
            quote::quote! { #range }.to_string()
        }
        _ => {
            // Fallback - use quote for other expressions
            quote::quote! { #expr }.to_string()
        }
    }
}

pub(crate) fn path_to_string(path: &syn::Path) -> String {
    quote::quote! { #path }.to_string()
}

/// Parse patterns that start with `=` token: `==` (equality) or `=~` (regex/like).
///
/// This utility handles the disambiguation and parsing of:
/// - `==` - Explicit equality comparison pattern (e.g., `== 42`)
/// - `=~` - Regex or Like pattern matching (e.g., `=~ r"pattern"` or `=~ expr`)
///
/// # Context
/// Called from `parse_pattern` after the caller has created a fork and verified
/// that the next token sequence starts with `=`. The caller creates the fork
/// (as that's the caller's responsibility), then this function performs the
/// lookahead to determine `==` vs `=~` and does the actual parsing.
///
/// # Performance Note
/// For `=~` patterns with string literals, the regex is compiled at macro
/// expansion time for better performance. Expression-based patterns use the
/// Like trait for runtime matching.
pub(crate) fn parse_eq_or_like(input: ParseStream) -> syn::Result<Pattern> {
    // Use peek2 to look ahead without forking (caller's fork verified we start with `=`)
    if input.peek2(Token![=]) {
        // This is `==` - explicit equality comparison
        return Ok(Pattern::Comparison(input.parse()?));
    }

    #[cfg(feature = "regex")]
    if input.peek2(Token![~]) {
        // Regex pattern matching with dual-path optimization
        let _: Token![=] = input.parse()?;
        let _: Token![~] = input.parse()?;

        // PERFORMANCE OPTIMIZATION: String literals are compiled at macro expansion time
        // This avoids runtime regex compilation for the common case
        let fork = input.fork();
        if let Ok(lit) = fork.parse::<syn::LitStr>() {
            // Example: `email: =~ r".*@example\.com"`
            // Compiles regex at macro expansion, fails early if invalid
            let parsed_lit = input.parse::<syn::LitStr>()?;
            return Ok(Pattern::Regex(PatternRegex {
                node_id: crate::parse::next_node_id(),
                pattern: lit.value(),
                span: parsed_lit.span(),
            }));
        } else {
            // Example: `email: =~ email_pattern` where email_pattern is a variable
            // Uses Like trait for runtime pattern matching
            let expr = input.parse::<syn::Expr>()?;
            return Ok(Pattern::Like(PatternLike {
                node_id: crate::parse::next_node_id(),
                expr,
            }));
        }
    }

    Err(input.error("expected `==` or `=~` pattern"))
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Pattern::Simple(p) => write!(f, "{}", p),
            Pattern::Struct(p) => write!(f, "{}", p),
            Pattern::Tuple(p) => write!(f, "{}", p),
            Pattern::Slice(p) => write!(f, "{}", p),
            Pattern::Comparison(p) => write!(f, "{} {}", p.op, expr_to_string(&p.expr)),
            Pattern::Range(p) => write!(f, "{}", p),
            #[cfg(feature = "regex")]
            Pattern::Regex(p) => write!(f, "{}", p),
            #[cfg(feature = "regex")]
            Pattern::Like(p) => write!(f, "{}", p),
            Pattern::Rest(p) => write!(f, "{}", p),
            Pattern::Wildcard(p) => write!(f, "{}", p),
            Pattern::Closure(p) => write!(f, "{}", p),
            Pattern::Map(p) => write!(f, "{}", p),
        }
    }
}
