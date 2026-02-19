//! Comparison pattern types and parsing.
//!
//! This module defines comparison operators and patterns like `> 30`, `<= 100`, etc.

use proc_macro2::Span;
use syn::{Token, parse::Parse, spanned::Spanned};

use crate::parse::next_node_id;

/// Comparison pattern: > 30, <= 100
#[derive(Debug, Clone)]
pub(crate) struct PatternComparison {
    pub node_id: usize,
    pub op: ComparisonOp,
    pub expr: syn::Expr,
}

#[derive(Debug, Clone)]
pub(crate) enum ComparisonOp {
    Less(Token![<]),
    LessEqual(Token![<=]),
    Greater(Token![>]),
    GreaterEqual(Token![>=]),
    Equal(Token![==]),
    NotEqual(Token![!=]),
}

impl ComparisonOp {
    pub fn span(&self) -> Span {
        match self {
            ComparisonOp::Less(t) => t.span(),
            ComparisonOp::LessEqual(t) => t.span(),
            ComparisonOp::Greater(t) => t.span(),
            ComparisonOp::GreaterEqual(t) => t.span(),
            ComparisonOp::Equal(t) => t.span(),
            ComparisonOp::NotEqual(t) => t.span(),
        }
    }
}

impl Parse for ComparisonOp {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Check compound operators before their single-char prefixes, since
        // peek(Token![<]) also matches the `<` in `<=`.
        if input.peek(Token![<=]) {
            Ok(ComparisonOp::LessEqual(input.parse()?))
        } else if input.peek(Token![<]) {
            Ok(ComparisonOp::Less(input.parse()?))
        } else if input.peek(Token![>=]) {
            Ok(ComparisonOp::GreaterEqual(input.parse()?))
        } else if input.peek(Token![>]) {
            Ok(ComparisonOp::Greater(input.parse()?))
        } else if input.peek(Token![==]) {
            Ok(ComparisonOp::Equal(input.parse()?))
        } else if input.peek(Token![!=]) {
            Ok(ComparisonOp::NotEqual(input.parse()?))
        } else {
            Err(input.error("expected comparison operator (<, <=, >, >=, ==, !=)"))
        }
    }
}

impl Parse for PatternComparison {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let op: ComparisonOp = input.parse()?;
        let expr: syn::Expr = input.parse()?;
        Ok(PatternComparison {
            node_id: next_node_id(),
            op,
            expr,
        })
    }
}
