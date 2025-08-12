use crate::{AssertStruct, ComparisonOp, Expected, FieldAssertion, Pattern};
use syn::{Result, Token, parse::Parse, parse::ParseStream, punctuated::Punctuated};
use std::cell::Cell;

thread_local! {
    static NODE_ID_COUNTER: Cell<usize> = Cell::new(0);
}

fn next_node_id() -> usize {
    NODE_ID_COUNTER.with(|counter| {
        let id = counter.get();
        counter.set(id + 1);
        id
    })
}

fn reset_node_counter() {
    NODE_ID_COUNTER.with(|counter| counter.set(0));
}

pub fn parse(input: proc_macro::TokenStream) -> syn::Result<AssertStruct> {
    // Reset the counter for each macro invocation
    reset_node_counter();
    syn::parse(input)
}

impl Parse for AssertStruct {
    /// Parses the top-level macro invocation.
    ///
    /// # Example Input
    /// ```text
    /// assert_struct!(value, Pattern { field: matcher, .. })
    /// assert_struct!(value, Some(> 30))
    /// assert_struct!(value, [1, 2, 3])
    /// ```
    ///
    /// The macro always expects: `expression`, `pattern`
    fn parse(input: ParseStream) -> Result<Self> {
        let value = input.parse()?;
        let _: Token![,] = input.parse()?;
        let pattern = parse_pattern(input)?;

        Ok(AssertStruct { value, pattern })
    }
}

impl Parse for Expected {
    /// Parses struct field patterns inside braces.
    ///
    /// # Example Input
    /// ```text
    /// // Inside: User { ... }
    /// name: "Alice", age: 30, ..
    /// name: "Bob", age: >= 18
    /// email: =~ r".*@example\.com", ..
    /// ```
    ///
    /// The `..` token enables partial matching - only specified fields are checked.
    fn parse(input: ParseStream) -> Result<Self> {
        let mut fields = Punctuated::new();
        let mut rest = false;

        while !input.is_empty() {
            // Check for rest pattern (..) which allows partial matching
            if input.peek(Token![..]) {
                let _: Token![..] = input.parse()?;
                rest = true;
                break;
            }

            fields.push_value(input.parse()?);

            if input.is_empty() {
                break;
            }

            let comma: Token![,] = input.parse()?;
            fields.push_punct(comma);

            // Rest pattern can appear after a comma
            if input.peek(Token![..]) {
                let _: Token![..] = input.parse()?;
                rest = true;
                break;
            }
        }

        Ok(Expected { fields, rest })
    }
}

/// Parse any pattern at any level - the heart of the macro's flexibility.
///
/// This function handles all pattern types in a specific order to avoid ambiguity.
/// The order matters because some patterns share prefixes (e.g., `..` vs `..n`).
fn parse_pattern(input: ParseStream) -> Result<Pattern> {
    // AMBIGUITY: `..` could be a rest pattern OR start of a range like `..10`
    // Example inputs:
    //   `..`        -> rest pattern (partial matching)
    //   `..10`      -> range pattern (exclusive upper bound)
    //   `..=10`     -> range pattern (inclusive upper bound)
    if input.peek(Token![..]) {
        let fork = input.fork();
        let _: Token![..] = fork.parse()?;

        // Distinguish by looking ahead after the `..`
        if fork.peek(Token![=]) || (!fork.is_empty() && !fork.peek(Token![,])) {
            // This is a range pattern like `..10` or `..=10`
            // Fall through to parse as expression later
        } else {
            // This is a rest pattern for partial matching
            let _: Token![..] = input.parse()?;
            return Ok(Pattern::Rest {
                node_id: next_node_id(),
            });
        }
    }

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
            return Ok(Pattern::Comparison {
                node_id: next_node_id(),
                op: ComparisonOp::LessEqual,
                expr: value,
            });
        } else {
            let value = input.parse()?;
            return Ok(Pattern::Comparison {
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
            return Ok(Pattern::Comparison {
                node_id: next_node_id(),
                op: ComparisonOp::GreaterEqual,
                expr: value,
            });
        } else {
            let value = input.parse()?;
            return Ok(Pattern::Comparison {
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
        if fork.parse::<Token![!]>().is_ok() && fork.peek(Token![=]) {
            let _: Token![!] = input.parse()?;
            let _: Token![=] = input.parse()?;
            let value = input.parse()?;
            return Ok(Pattern::Comparison {
                node_id: next_node_id(),
                op: ComparisonOp::NotEqual,
                expr: value,
            });
        }
    }

    // `=` could start `==` (equality) or `=~` (regex pattern)
    if input.peek(Token![=]) {
        let fork = input.fork();
        if fork.parse::<Token![=]>().is_ok() {
            if fork.peek(Token![=]) {
                // Explicit equality check
                // Example: `status: == "ok"`
                let _: Token![=] = input.parse()?;
                let _: Token![=] = input.parse()?;
                let value = input.parse()?;
                return Ok(Pattern::Comparison {
                    node_id: next_node_id(),
                    op: ComparisonOp::Equal,
                    expr: value,
                });
            }
            #[cfg(feature = "regex")]
            if fork.peek(Token![~]) {
                // Regex pattern matching with dual-path optimization
                let _: Token![=] = input.parse()?;
                let _: Token![~] = input.parse()?;

                // PERFORMANCE OPTIMIZATION: String literals are compiled at macro expansion time
                // This avoids runtime regex compilation for the common case
                let fork = input.fork();
                if let Ok(lit) = fork.parse::<syn::LitStr>() {
                    // Example: `email: =~ r".*@example\.com"`
                    // Compiles regex at macro expansion, fails early if invalid
                    input.parse::<syn::LitStr>()?;
                    return Ok(Pattern::Regex {
                        node_id: next_node_id(),
                        pattern: lit.value(),
                    });
                } else {
                    // Example: `email: =~ email_pattern` where email_pattern is a variable
                    // Uses Like trait for runtime pattern matching
                    let expr = input.parse::<syn::Expr>()?;
                    return Ok(Pattern::Like {
                        node_id: next_node_id(),
                        expr,
                    });
                }
            }
        }
    }

    // Slice patterns for Vec/array matching
    // Example: `[1, 2, 3]` or `[> 0, < 10, == 5]`
    if input.peek(syn::token::Bracket) {
        let content;
        syn::bracketed!(content in input);
        let elements = parse_pattern_list(&content)?;
        return Ok(Pattern::Slice {
            node_id: next_node_id(),
            elements,
        });
    }

    // Standalone tuple pattern (no type prefix)
    // Example: `(10, 20)` or `(> 10, < 30)`
    if input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in input);
        let elements = parse_pattern_list(&content)?;
        return Ok(Pattern::Tuple {
            node_id: next_node_id(),
            path: None,
            elements,
        });
    }

    // Complex path-based patterns: structs, enums, tuple variants
    // This is where disambiguation becomes critical
    let fork = input.fork();
    if let Ok(path) = fork.parse::<syn::Path>() {
        // Path followed by braces is a struct pattern
        // Example: `User { name: "Alice", age: 30 }`
        if fork.peek(syn::token::Brace) {
            let path: syn::Path = input.parse()?;
            let content;
            syn::braced!(content in input);
            let expected: Expected = content.parse()?;
            return Ok(Pattern::Struct {
                node_id: next_node_id(),
                path,
                fields: expected.fields,
                rest: expected.rest,
            });
        }

        // Path followed by parens could be:
        // 1. Enum with patterns: `Some(> 30)` - needs special parsing
        // 2. Simple expression: `Some(value)` - parse as single expression
        if fork.peek(syn::token::Paren) {
            let path: syn::Path = input.parse()?;
            let content;
            syn::parenthesized!(content in input);

            // CRITICAL DISAMBIGUATION: Is this `Some(> 30)` or `Some(my_var)`?
            // We need to check if the content has special pattern syntax
            let fork = content.fork();
            let has_special = check_for_special_syntax(&fork);

            if has_special {
                // Contains pattern syntax like `>`, `==`, nested patterns
                // Example: `Some(> 30)`, `Event::Click(>= 0, < 100)`
                let elements = parse_pattern_list(&content)?;
                return Ok(Pattern::Tuple {
                    node_id: next_node_id(),
                    path: Some(path),
                    elements,
                });
            } else {
                // Simple expression without pattern syntax
                // Example: `Some(expected_value)`, `Ok(result)`
                // We treat the whole content as a single expression
                let expr = content.parse()?;
                return Ok(Pattern::Tuple {
                    node_id: next_node_id(),
                    path: Some(path),
                    elements: vec![Pattern::Simple {
                        node_id: next_node_id(),
                        expr,
                    }],
                });
            }
        }

        // Unit variants (no parens or braces)
        // Heuristic: If it starts with uppercase, likely an enum variant
        // Examples: `None`, `Status::Active`, `Color::Red`
        if let Some(segment) = path.segments.last() {
            let name = segment.ident.to_string();
            if name.chars().next().is_some_and(|c| c.is_uppercase()) {
                let path: syn::Path = input.parse()?;
                return Ok(Pattern::Tuple {
                    node_id: next_node_id(),
                    path: Some(path),
                    elements: vec![],
                });
            }
        }
    }

    // Everything else is either a simple expression or range
    let expr: syn::Expr = input.parse()?;

    // Range expressions like `18..65` or `0.0..100.0`
    if matches!(expr, syn::Expr::Range(_)) {
        Ok(Pattern::Range {
            node_id: next_node_id(),
            expr,
        })
    } else {
        // Simple value or expression
        // Examples: `42`, `"hello"`, `my_variable`, `compute_value()`
        Ok(Pattern::Simple {
            node_id: next_node_id(),
            expr,
        })
    }
}

/// Parse a comma-separated list of patterns.
/// Used inside tuples, slices, and enum variants.
fn parse_pattern_list(input: ParseStream) -> Result<Vec<Pattern>> {
    let mut patterns = Vec::new();

    while !input.is_empty() {
        patterns.push(parse_pattern(input)?);

        if !input.is_empty() {
            let _: Token![,] = input.parse()?;
        }
    }

    Ok(patterns)
}

impl Parse for FieldAssertion {
    /// Parses a single field assertion within a struct pattern.
    ///
    /// # Example Input
    /// ```text
    /// name: "Alice"
    /// age: >= 18
    /// email: =~ r".*@example\.com"
    /// ```
    fn parse(input: ParseStream) -> Result<Self> {
        let field_name: syn::Ident = input.parse()?;
        let _: Token![:] = input.parse()?;
        let pattern = parse_pattern(input)?;

        Ok(FieldAssertion {
            field_name,
            pattern,
        })
    }
}

/// Critical disambiguation function that determines whether parenthesized content
/// contains special pattern syntax or is just a simple expression.
///
/// This solves the ambiguity between:
/// - `Some(> 30)` - contains pattern syntax, needs special parsing
/// - `Some(my_var)` - simple expression, parse as-is
/// - `Some((true, false))` - tuple expression, parse as-is
/// - `Event::Click(>= 0, < 100)` - multiple patterns, needs special parsing
///
/// The fork-and-peek pattern is essential here - we look ahead without
/// consuming tokens to make the decision.
fn check_for_special_syntax(content: ParseStream) -> bool {
    // Comparison operators indicate pattern syntax
    if content.peek(Token![<]) || content.peek(Token![>]) {
        return true;
    }

    // Check for != operator (but not just ! which could be boolean negation)
    if content.peek(Token![!]) {
        let fork = content.fork();
        if fork.parse::<Token![!]>().is_ok() && fork.peek(Token![=]) {
            return true;
        }
    }

    // Check for == or =~ operators
    if content.peek(Token![=]) {
        let fork = content.fork();
        if fork.parse::<Token![=]>().is_ok() && (fork.peek(Token![=]) || fork.peek(Token![~])) {
            return true;
        }
    }

    // Nested slice patterns like `Some([1, 2, 3])`
    if content.peek(syn::token::Bracket) {
        return true;
    }

    // Nested struct/enum patterns like `Some(User { ... })`
    let fork = content.fork();
    if let Ok(_path) = fork.parse::<syn::Path>() {
        if fork.peek(syn::token::Brace) || fork.peek(syn::token::Paren) {
            return true;
        }
    }

    // Multiple comma-separated elements indicate tuple pattern
    // BUT: Be careful! `(true, false)` is a valid tuple expression
    // We only treat it as special if it would contain patterns
    let fork = content.fork();
    if fork.parse::<syn::Expr>().is_ok() && fork.peek(Token![,]) {
        return true;
    }

    false
}
