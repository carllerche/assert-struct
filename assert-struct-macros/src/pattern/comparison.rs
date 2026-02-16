//! Comparison pattern types and parsing.
//!
//! This module defines comparison operators and patterns like `> 30`, `<= 100`, etc.

use std::fmt;
use syn::{Token, parse::Parse};

use crate::parse::next_node_id;

/// Comparison pattern: > 30, <= 100
#[derive(Debug, Clone)]
pub(crate) struct PatternComparison {
    pub node_id: usize,
    pub op: ComparisonOp,
    pub expr: syn::Expr,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum ComparisonOp {
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equal,
    NotEqual,
}

impl fmt::Display for ComparisonOp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ComparisonOp::Less => write!(f, "<"),
            ComparisonOp::LessEqual => write!(f, "<="),
            ComparisonOp::Greater => write!(f, ">"),
            ComparisonOp::GreaterEqual => write!(f, ">="),
            ComparisonOp::Equal => write!(f, "=="),
            ComparisonOp::NotEqual => write!(f, "!="),
        }
    }
}

impl Parse for PatternComparison {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Comparison operators are checked early to capture them before
        // they could be parsed as part of an expression
        // Examples:
        //   `< 100`     -> less than 100
        //   `>= 18`     -> greater than or equal to 18
        //   `> compute_threshold()` -> comparison with function result
        if input.peek(Token![<]) {
            let _: Token![<] = input.parse()?;
            if input.peek(Token![=]) {
                let _: Token![=] = input.parse()?;
                let value = input.parse()?;
                return Ok(PatternComparison {
                    node_id: next_node_id(),
                    op: ComparisonOp::LessEqual,
                    expr: value,
                });
            } else {
                let value = input.parse()?;
                return Ok(PatternComparison {
                    node_id: next_node_id(),
                    op: ComparisonOp::Less,
                    expr: value,
                });
            }
        }

        if input.peek(Token![>]) {
            let _: Token![>] = input.parse()?;
            if input.peek(Token![=]) {
                let _: Token![=] = input.parse()?;
                let value = input.parse()?;
                return Ok(PatternComparison {
                    node_id: next_node_id(),
                    op: ComparisonOp::GreaterEqual,
                    expr: value,
                });
            } else {
                let value = input.parse()?;
                return Ok(PatternComparison {
                    node_id: next_node_id(),
                    op: ComparisonOp::Greater,
                    expr: value,
                });
            }
        }

        // `!=` needs special handling because `!` could start other expressions
        // Example: `!= "error"` vs `!flag` (not pattern vs boolean negation)
        if input.peek(Token![!]) {
            let fork = input.fork();
            let _: Token![!] = fork.parse()?;
            if fork.peek(Token![=]) {
                // Actually consume the tokens from input
                let _: Token![!] = input.parse()?;
                let _: Token![=] = input.parse()?;
                let value = input.parse()?;
                return Ok(PatternComparison {
                    node_id: next_node_id(),
                    op: ComparisonOp::NotEqual,
                    expr: value,
                });
            }
        }

        // `==` for explicit equality
        // Example: `== 42` vs just `42` (both mean the same thing, but == is explicit)
        if input.peek(Token![==]) {
            let _: Token![==] = input.parse()?;
            let value = input.parse()?;
            return Ok(PatternComparison {
                node_id: next_node_id(),
                op: ComparisonOp::Equal,
                expr: value,
            });
        }

        Err(input.error("expected comparison operator (<, <=, >, >=, ==, !=)"))
    }
}
