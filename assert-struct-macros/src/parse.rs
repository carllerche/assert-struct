use crate::AssertStruct;
use crate::pattern::{Pattern, PatternRange, PatternSimple, PatternTuple, TupleElement};
use std::cell::Cell;
use syn::{Result, Token, parse::Parse, parse::ParseStream};

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
        // Check if this is a wildcard struct pattern: `_ { ... }`
        if input.peek2(syn::token::Brace) {
            return Ok(Pattern::Struct(input.parse()?));
        } else {
            // Regular wildcard pattern
            return Ok(Pattern::Wildcard(input.parse()?));
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

        // Parse as tuple pattern with element-wise matching
        // Example: `(> 10, < 30)`, `(== 5, != 10)`
        let elements = TupleElement::parse_comma_separated(&content)?;
        return Ok(Pattern::Tuple(PatternTuple {
            node_id: next_node_id(),
            path: None,
            elements,
        }));
    }

    // Complex path-based patterns: structs, enums, tuple variants
    // This is where disambiguation becomes critical
    let fork = input.fork();
    if let Ok(path) = fork.parse::<syn::Path>() {
        // Path followed by braces is a struct pattern
        // Example: `User { name: "Alice", age: 30 }`
        if fork.peek(syn::token::Brace) {
            return Ok(Pattern::Struct(input.parse()?));
        }

        // Path followed by parens is an enum/tuple variant with patterns
        // Example: `Some(> 30)`, `Event::Click(>= 0, < 100)`
        if fork.peek(syn::token::Paren) {
            let path: syn::Path = input.parse()?;
            let content;
            syn::parenthesized!(content in input);

            // Parse tuple elements with pattern syntax
            // Example: `Some(> 30)`, `Event::Click(>= 0, < 100)`
            let elements = TupleElement::parse_comma_separated(&content)?;
            return Ok(Pattern::Tuple(PatternTuple {
                node_id: next_node_id(),
                path: Some(path),
                elements,
            }));
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

