//! Pattern types for structural assertions.
//!
//! This module defines the various pattern types that can be used in assertions,
//! along with helper types for field operations and tuple elements.

mod closure;
mod comparison;
mod field;
mod map;
mod range;
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
pub(crate) use simple::PatternSimple;
pub(crate) use slice::PatternSlice;
pub(crate) use struct_pattern::PatternStruct;
pub(crate) use tuple::{PatternTuple, TupleElement};
pub(crate) use wildcard::PatternWildcard;

#[cfg(feature = "regex")]
pub(crate) use regex::{PatternLike, PatternRegex};

use std::fmt;

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
            Pattern::Wildcard(p) => write!(f, "{}", p),
            Pattern::Closure(p) => write!(f, "{}", p),
            Pattern::Map(p) => write!(f, "{}", p),
        }
    }
}
