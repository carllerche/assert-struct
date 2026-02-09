use crate::pattern::{
    FieldOperation, Pattern, PatternRange, PatternRest,
    PatternSimple, PatternStruct, PatternTuple, PatternWildcard, TupleElement,
};
use crate::{AssertStruct, Expected};
use std::cell::Cell;
use syn::{Result, Token, parse::Parse, parse::ParseStream, punctuated::Punctuated};

thread_local! {
    static NODE_ID_COUNTER: Cell<usize> = const { Cell::new(0) };
}

pub(crate) fn next_node_id() -> usize {
    NODE_ID_COUNTER.with(|counter| {
        let id = counter.get();
        counter.set(id + 1);
        id
    })
}

fn reset_node_counter() {
    NODE_ID_COUNTER.with(|counter| counter.set(0));
}

pub(crate) fn parse(input: proc_macro::TokenStream) -> syn::Result<AssertStruct> {
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
pub(crate) fn parse_pattern(input: ParseStream) -> Result<Pattern> {
    // Closure pattern: |x| expr or move |x| expr for custom validation (escape hatch)
    // Examples: `|x| x > 5`, `move |x| complex_logic(x)`, `|x| { x.len() > 0 }`
    if input.peek(Token![|]) || (input.peek(Token![move]) && input.peek2(Token![|])) {
        return Ok(Pattern::Closure(input.parse()?));
    }

    // Wildcard pattern: _ for ignoring a value while asserting it exists
    // Example: `Some(_)`, `field: _`, `[1, _, 3]`
    // Special case: `_ { ... }` for wildcard struct patterns
    if input.peek(Token![_]) {
        let fork = input.fork();
        let _: Token![_] = fork.parse()?;

        // Check if this is a wildcard struct pattern: `_ { ... }`
        if fork.peek(syn::token::Brace) {
            let underscore_token: Token![_] = input.parse()?;
            let content;
            syn::braced!(content in input);
            let expected: Expected = content.parse()?;

            // Wildcard struct patterns must use rest pattern (..)
            // to indicate partial matching
            if !expected.rest {
                return Err(syn::Error::new_spanned(
                    underscore_token,
                    "Wildcard struct patterns must use '..' for partial matching",
                ));
            }

            return Ok(Pattern::Struct(PatternStruct {
                node_id: next_node_id(),
                path: None, // None indicates wildcard
                fields: expected.fields,
                rest: expected.rest,
            }));
        } else {
            // Regular wildcard pattern
            let _: Token![_] = input.parse()?;
            return Ok(Pattern::Wildcard(PatternWildcard {
                node_id: next_node_id(),
            }));
        }
    }

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
            return Ok(Pattern::Rest(PatternRest {
                node_id: next_node_id(),
            }));
        }
    }

    // Try to parse as a comparison pattern (<, <=, >, >=, ==, !=)
    // Use fork to check if this looks like a comparison without consuming tokens
    if input.peek(Token![<]) || input.peek(Token![>]) || input.peek(Token![!]) {
        // These always start comparisons, safe to parse directly
        return Ok(Pattern::Comparison(input.parse()?));
    }

    // `=` could start `==` (equality) or `=~` (regex pattern)
    if input.peek(Token![=]) {
        return crate::pattern::parse_eq_or_like(input);
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
        let content;
        syn::parenthesized!(content in input);

        // Check for special syntax to distinguish patterns from simple expressions
        let fork = content.fork();
        let has_special = check_for_special_syntax(&fork);

        if has_special {
            // Contains pattern syntax like `>`, `==`, nested patterns
            // Example: `(> 10, < 30)`, `(== 5, != 10)`
            let elements = parse_tuple_elements(&content)?;
            return Ok(Pattern::Tuple(PatternTuple {
                node_id: next_node_id(),
                path: None,
                elements,
            }));
        } else {
            // Simple expression without pattern syntax
            // Example: `(10, 20)`, `(expected_x, expected_y)`
            // Treat as a single simple expression
            let expr = content.parse()?;
            return Ok(Pattern::Simple(PatternSimple {
                node_id: next_node_id(),
                expr,
            }));
        }
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
            return Ok(Pattern::Struct(PatternStruct {
                node_id: next_node_id(),
                path: Some(path),
                fields: expected.fields,
                rest: expected.rest,
            }));
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
                let elements = parse_tuple_elements(&content)?;
                return Ok(Pattern::Tuple(PatternTuple {
                    node_id: next_node_id(),
                    path: Some(path),
                    elements,
                }));
            } else {
                // Simple expression without pattern syntax
                // Example: `Some(expected_value)`, `Ok(result)`
                // We treat the whole content as a single expression
                let expr = content.parse()?;
                return Ok(Pattern::Tuple(PatternTuple {
                    node_id: next_node_id(),
                    path: Some(path),
                    elements: vec![TupleElement::Positional {
                        pattern: Pattern::Simple(PatternSimple {
                            node_id: next_node_id(),
                            expr,
                        }),
                    }],
                }));
            }
        }

        // Unit variants (no parens or braces)
        // Heuristic: If it starts with uppercase, likely an enum variant
        // Examples: `None`, `Status::Active`, `Color::Red`
        if let Some(segment) = path.segments.last() {
            let name = segment.ident.to_string();
            if name.chars().next().is_some_and(|c| c.is_uppercase()) {
                let path: syn::Path = input.parse()?;
                return Ok(Pattern::Tuple(PatternTuple {
                    node_id: next_node_id(),
                    path: Some(path),
                    elements: vec![],
                }));
            }
        }
    }

    // Everything else is either a simple expression or range
    let expr: syn::Expr = input.parse()?;

    // Range expressions like `18..65` or `0.0..100.0`
    if matches!(expr, syn::Expr::Range(_)) {
        Ok(Pattern::Range(PatternRange {
            node_id: next_node_id(),
            expr,
        }))
    } else {
        // Simple value or expression
        // Examples: `42`, `"hello"`, `my_variable`, `compute_value()`
        Ok(Pattern::Simple(PatternSimple {
            node_id: next_node_id(),
            expr,
        }))
    }
}




/// Parse a comma-separated list of tuple elements, supporting both positional and indexed syntax.
/// Used inside tuple patterns to handle mixed syntax like ("foo", *1: "bar", "baz")
fn parse_tuple_elements(input: ParseStream) -> Result<Vec<TupleElement>> {
    let mut elements = Vec::new();
    let mut position = 0;

    while !input.is_empty() {
        // First, try to parse operations (like * for deref)
        let operations = FieldOperation::parse_option(input)?;

        // Check if this is an indexed element by looking for number followed by colon or method call
        let fork = input.fork();
        let is_indexed = if let Ok(_index_lit) = fork.parse::<syn::LitInt>() {
            fork.peek(Token![:]) || fork.peek(Token![.])
        } else {
            false
        };

        if is_indexed {
            // Parse indexed element: index: pattern, *index: pattern, or index.method(): pattern
            let index_lit: syn::LitInt = input.parse()?;
            let index: usize = index_lit.base10_parse()?;

            // Validate that index matches current position
            if index != position {
                return Err(syn::Error::new_spanned(
                    index_lit,
                    format!("Index {} must match position {} in tuple", index, position),
                ));
            }

            // Check for method calls after the index: 0.len():
            let final_operations = if input.peek(Token![.]) {
                Some(FieldOperation::parse_chain(input, operations)?)
            } else {
                operations
            };

            let _: Token![:] = input.parse()?;
            let pattern = parse_pattern(input)?;

            elements.push(TupleElement::Indexed {
                index,
                operations: final_operations,
                pattern,
            });
        } else {
            // If we parsed operations but no index, this is an error
            if operations.is_some() {
                return Err(syn::Error::new(
                    input.span(),
                    "Operations like * can only be used with indexed elements (e.g., *0:, *1:)",
                ));
            }

            // Parse positional element: just a pattern
            let pattern = parse_pattern(input)?;
            elements.push(TupleElement::Positional { pattern });
        }

        position += 1;

        if !input.is_empty() {
            let _: Token![,] = input.parse()?;
        }
    }

    Ok(elements)
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
    // Wildcard pattern
    if content.peek(Token![_]) {
        return true;
    }

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

    // Check for indexed elements and method calls: `0:`, `1.method():`
    let fork = content.fork();
    if let Ok(_index_lit) = fork.parse::<syn::LitInt>() {
        if fork.peek(Token![:]) || fork.peek(Token![.]) {
            return true;
        }
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

