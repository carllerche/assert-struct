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
mod set;
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
pub(crate) use set::PatternSet;
pub(crate) use simple::PatternSimple;
pub(crate) use slice::PatternSlice;
pub(crate) use string::PatternString;
pub(crate) use struct_pattern::PatternStruct;
pub(crate) use tuple::{PatternTuple, TupleElement};
pub(crate) use wildcard::PatternWildcard;

#[cfg(feature = "regex")]
pub(crate) use regex::{PatternLike, PatternRegex};

use proc_macro2::Span;
use syn::{
    Token,
    parse::{Parse, ParseStream},
    spanned::Spanned,
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
    Set(PatternSet),
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

impl Pattern {
    pub(crate) fn span(&self) -> Option<Span> {
        match self {
            Pattern::Simple(PatternSimple { expr, .. }) => Some(expr.span()),
            Pattern::String(PatternString { lit, .. }) => Some(lit.span()),
            Pattern::Comparison(PatternComparison { op, expr, .. }) => {
                let op_span = op.span();
                Some(op_span.join(expr.span()).unwrap_or(op_span))
            }
            Pattern::Range(PatternRange { expr, .. }) => Some(expr.span()),
            #[cfg(feature = "regex")]
            Pattern::Regex(PatternRegex { span, .. }) => Some(*span),
            #[cfg(feature = "regex")]
            Pattern::Like(PatternLike { expr, .. }) => Some(expr.span()),
            Pattern::Struct(PatternStruct { path, .. }) => path.as_ref().map(|p| p.span()),
            Pattern::Enum(PatternEnum { path, .. }) => Some(path.span()),
            Pattern::Tuple(PatternTuple { .. })
            | Pattern::Slice(PatternSlice { .. })
            | Pattern::Set(PatternSet { .. })
            | Pattern::Wildcard(PatternWildcard { .. })
            | Pattern::Map(PatternMap { .. }) => None,
            Pattern::Closure(PatternClosure { closure, .. }) => Some(closure.span()),
        }
    }

    /// Compute the source location for the pattern's anchor token(s).
    ///
    /// Returns `(line_start, col_start, line_end, col_end)` where lines are 1-indexed
    /// and columns are 0-indexed (proc_macro2 convention). Returns `(0, 0, 0, 0)` for
    /// patterns without a meaningful source location.
    ///
    /// Computes start and end from constituent tokens independently to avoid
    /// `Span::join()`, which is nightly-only in proc_macro context. For example,
    /// a `Comparison` like `> 30` uses the operator for the start and the expression
    /// for the end. Enum and struct paths highlight only the type path, not the
    /// arguments or fields that follow.
    pub(crate) fn location(&self) -> (u32, u32, u32, u32) {
        match self {
            Pattern::Simple(PatternSimple { expr, .. }) => {
                let start = expr.span().start();
                let end = expr.span().end();
                (
                    start.line as u32,
                    start.column as u32,
                    end.line as u32,
                    end.column as u32,
                )
            }
            Pattern::String(PatternString { lit, .. }) => {
                let start = lit.span().start();
                let end = lit.span().end();
                (
                    start.line as u32,
                    start.column as u32,
                    end.line as u32,
                    end.column as u32,
                )
            }
            Pattern::Comparison(PatternComparison { op, expr, .. }) => {
                let start = op.span().start();
                let end = expr.span().end();
                (
                    start.line as u32,
                    start.column as u32,
                    end.line as u32,
                    end.column as u32,
                )
            }
            Pattern::Range(PatternRange { expr, .. }) => {
                if let syn::Expr::Range(range_expr) = expr {
                    let start = range_expr
                        .start
                        .as_ref()
                        .map(|s| s.span().start())
                        .unwrap_or_else(|| range_expr.limits.span().start());
                    let end = range_expr
                        .end
                        .as_ref()
                        .map(|e| e.span().end())
                        .unwrap_or_else(|| range_expr.limits.span().end());
                    (
                        start.line as u32,
                        start.column as u32,
                        end.line as u32,
                        end.column as u32,
                    )
                } else {
                    let start = expr.span().start();
                    let end = expr.span().end();
                    (
                        start.line as u32,
                        start.column as u32,
                        end.line as u32,
                        end.column as u32,
                    )
                }
            }
            #[cfg(feature = "regex")]
            Pattern::Regex(PatternRegex { span, .. }) => {
                let start = span.start();
                let end = span.end();
                (
                    start.line as u32,
                    start.column as u32,
                    end.line as u32,
                    end.column as u32,
                )
            }
            #[cfg(feature = "regex")]
            Pattern::Like(PatternLike { expr, .. }) => {
                let start = expr.span().start();
                let end = expr.span().end();
                (
                    start.line as u32,
                    start.column as u32,
                    end.line as u32,
                    end.column as u32,
                )
            }
            // Highlight only the path itself, not the args or fields that follow.
            // Use first/last segment idents independently to avoid Span::join().
            Pattern::Struct(PatternStruct {
                path: Some(path), ..
            })
            | Pattern::Enum(PatternEnum { path, .. }) => {
                let first = path.segments.first().map(|s| s.ident.span().start());
                let last = path.segments.last().map(|s| s.ident.span().end());
                match (first, last) {
                    (Some(start), Some(end)) => (
                        start.line as u32,
                        start.column as u32,
                        end.line as u32,
                        end.column as u32,
                    ),
                    _ => (0, 0, 0, 0),
                }
            }
            Pattern::Closure(PatternClosure { closure, .. }) => {
                let start = closure.span().start();
                let end = closure.span().end();
                (
                    start.line as u32,
                    start.column as u32,
                    end.line as u32,
                    end.column as u32,
                )
            }
            Pattern::Tuple(PatternTuple { span, .. })
            | Pattern::Slice(PatternSlice { span, .. })
            | Pattern::Set(PatternSet { span, .. })
            | Pattern::Map(PatternMap { span, .. }) => {
                let start = span.start();
                let end = span.end();
                (
                    start.line as u32,
                    start.column as u32,
                    end.line as u32,
                    end.column as u32,
                )
            }
            // Wildcard struct patterns and plain wildcards have no meaningful location.
            Pattern::Struct(PatternStruct { path: None, .. }) | Pattern::Wildcard(_) => {
                (0, 0, 0, 0)
            }
        }
    }
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

        // Set pattern for unordered collection matching
        // Example: `#(1, 2, 3)` or `#(> 0, < 10, ..)`
        if input.peek(Token![#]) && input.peek2(syn::token::Paren) {
            return Ok(Pattern::Set(input.parse()?));
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
