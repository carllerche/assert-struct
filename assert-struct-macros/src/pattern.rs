//! Pattern types for structural assertions.
//!
//! This module defines the various pattern types that can be used in assertions,
//! along with helper types for field operations and tuple elements.

mod closure;
mod comparison;
mod enum_pattern;
mod field;
mod map;
mod range;
mod simple;
mod slice;
mod string;
mod struct_pattern;
mod tuple;
mod wildcard;

#[cfg(feature = "regex")]
mod regex;

// Re-export all pattern types
pub(crate) use closure::PatternClosure;
pub(crate) use comparison::{ComparisonOp, PatternComparison};
pub(crate) use enum_pattern::PatternEnum;
pub(crate) use field::{FieldAssertion, FieldOperation};
pub(crate) use map::PatternMap;
pub(crate) use range::PatternRange;
pub(crate) use simple::PatternSimple;
pub(crate) use slice::PatternSlice;
pub(crate) use string::PatternString;
pub(crate) use struct_pattern::PatternStruct;
pub(crate) use tuple::{PatternTuple, TupleElement};
pub(crate) use wildcard::PatternWildcard;

#[cfg(feature = "regex")]
pub(crate) use regex::{PatternLike, PatternRegex};

use std::fmt;

use syn::{
    Token,
    parse::{Parse, ParseStream},
};

/// Unified pattern type that can represent any pattern
#[derive(Debug, Clone)]
pub(crate) enum Pattern {
    Simple(PatternSimple),
    String(PatternString),
    Struct(PatternStruct),
    Enum(PatternEnum),
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

impl Parse for Pattern {
    /// Parse any pattern at any level - the heart of the macro's flexibility.
    ///
    /// This handles all pattern types in a specific order to avoid ambiguity.
    /// The order matters because some patterns share prefixes.
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Closure pattern: |x| expr or move |x| expr for custom validation (escape hatch)
        // Examples: `|x| x > 5`, `move |x| complex_logic(x)`, `|x| { x.len() > 0 }`
        if input.peek(Token![|]) || (input.peek(Token![move]) && input.peek2(Token![|])) {
            return Ok(Pattern::Closure(input.parse()?));
        }

        // Wildcard pattern: _ for ignoring a value while asserting it exists
        // Example: `Some(_)`, `field: _`, `[1, _, 3]`
        // Special case: `_ { ... }` for wildcard struct patterns
        if input.peek(Token![_]) {
            // Check if this is a wildcard struct pattern: `_ { ... }`
            if input.peek2(syn::token::Brace) {
                return Ok(Pattern::Struct(input.parse()?));
            } else {
                // Regular wildcard pattern
                return Ok(Pattern::Wildcard(input.parse()?));
            }
        }

        // Try to parse as a comparison pattern (<, <=, >, >=, ==, !=)
        if input.peek(Token![<]) || input.peek(Token![>]) || input.peek(Token![!]) {
            // These always start comparisons, safe to parse directly
            return Ok(Pattern::Comparison(input.parse()?));
        }

        // `=` could start `==` (equality) or `=~` (regex pattern)
        if input.peek(Token![=]) {
            if input.peek2(Token![=]) {
                // This is `==` - explicit equality comparison
                return Ok(Pattern::Comparison(input.parse()?));
            }

            #[cfg(feature = "regex")]
            if input.peek2(Token![~]) {
                // This is `=~` - regex/like pattern
                let pattern: PatternLike = input.parse()?;
                return Ok(pattern.into_pattern());
            }

            return Err(input.error("expected `==` or `=~` pattern"));
        }

        // Map patterns for map-like structures using duck typing
        // Example: `#{ "key": "value" }` or `#{ "key": > 5, .. }`
        if input.peek(Token![#]) && input.peek2(syn::token::Brace) {
            return Ok(Pattern::Map(input.parse()?));
        }

        // Slice patterns for Vec/array matching
        // Example: `[1, 2, 3]` or `[> 0, < 10, == 5]`
        if input.peek(syn::token::Bracket) {
            return Ok(Pattern::Slice(input.parse()?));
        }

        // Standalone tuple pattern (no type prefix)
        // Example: `(10, 20)` or `(> 10, < 30)`
        if input.peek(syn::token::Paren) {
            return Ok(Pattern::Tuple(input.parse()?));
        }

        // Complex path-based patterns: structs, enums, tuple variants
        // This is where disambiguation becomes critical
        let fork = input.fork();
        if fork.parse::<syn::Path>().is_ok() {
            // Path followed by braces is a struct pattern
            // Example: `User { name: "Alice", age: 30 }`
            if fork.peek(syn::token::Brace) {
                return Ok(Pattern::Struct(input.parse()?));
            }

            // Path followed by parens OR standalone path is an enum variant
            // Example: `Some(> 30)`, `Event::Click(>= 0, < 100)`, `Status::Active`
            return Ok(Pattern::Enum(input.parse()?));
        }

        // Everything else is either a range, string literal, or simple expression
        // Try range first, then string literal, then fallback to simple
        let fork = input.fork();
        if fork.parse::<PatternRange>().is_ok() {
            // Range expressions like `18..65` or `0.0..100.0`
            Ok(Pattern::Range(input.parse()?))
        } else if input.peek(syn::LitStr) {
            // String literal: "hello", "world"
            Ok(Pattern::String(PatternString::new(input.parse()?)))
        } else {
            // Simple value or expression
            // Examples: `42`, `true`, `my_variable`, `compute_value()`
            Ok(Pattern::Simple(input.parse()?))
        }
    }
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
            Pattern::String(p) => write!(f, "{}", p),
            Pattern::Struct(p) => write!(f, "{}", p),
            Pattern::Enum(p) => write!(f, "{}", p),
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
