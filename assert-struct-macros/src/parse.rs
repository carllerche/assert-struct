use crate::{AssertStruct, ComparisonOp, Expected, FieldAssertion, Pattern};
use syn::{Result, Token, parse::Parse, parse::ParseStream, punctuated::Punctuated};

pub fn parse(input: proc_macro::TokenStream) -> syn::Result<AssertStruct> {
    syn::parse(input)
}

impl Parse for AssertStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let value = input.parse()?;
        let _: Token![,] = input.parse()?;
        let pattern = parse_pattern(input)?;

        Ok(AssertStruct { value, pattern })
    }
}

impl Parse for Expected {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut fields = Punctuated::new();
        let mut rest = false;

        while !input.is_empty() {
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

            if input.peek(Token![..]) {
                let _: Token![..] = input.parse()?;
                rest = true;
                break;
            }
        }

        Ok(Expected { fields, rest })
    }
}

// Parse any pattern at any level
fn parse_pattern(input: ParseStream) -> Result<Pattern> {
    // Check for .. but need to distinguish from range patterns (..n, ..=n)
    if input.peek(Token![..]) {
        let fork = input.fork();
        let _: Token![..] = fork.parse()?;

        // Check if this is a range pattern (..n or ..=n)
        if fork.peek(Token![=]) || (!fork.is_empty() && !fork.peek(Token![,])) {
            // This is a range pattern, fall through to parse as expression
        } else {
            // This is a rest pattern
            let _: Token![..] = input.parse()?;
            return Ok(Pattern::Rest);
        }
    }

    // Check for comparison operators: <, <=, >, >=
    if input.peek(Token![<]) {
        let _: Token![<] = input.parse()?;
        if input.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            let value = input.parse()?;
            return Ok(Pattern::Comparison(ComparisonOp::LessEqual, value));
        } else {
            let value = input.parse()?;
            return Ok(Pattern::Comparison(ComparisonOp::Less, value));
        }
    }

    if input.peek(Token![>]) {
        let _: Token![>] = input.parse()?;
        if input.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            let value = input.parse()?;
            return Ok(Pattern::Comparison(ComparisonOp::GreaterEqual, value));
        } else {
            let value = input.parse()?;
            return Ok(Pattern::Comparison(ComparisonOp::Greater, value));
        }
    }

    // Check for != operator
    if input.peek(Token![!]) {
        let fork = input.fork();
        if fork.parse::<Token![!]>().is_ok() && fork.peek(Token![=]) {
            let _: Token![!] = input.parse()?;
            let _: Token![=] = input.parse()?;
            let value = input.parse()?;
            return Ok(Pattern::Comparison(ComparisonOp::NotEqual, value));
        }
    }

    // Check for == or =~ operators
    if input.peek(Token![=]) {
        let fork = input.fork();
        if fork.parse::<Token![=]>().is_ok() {
            if fork.peek(Token![=]) {
                // == operator
                let _: Token![=] = input.parse()?;
                let _: Token![=] = input.parse()?;
                let value = input.parse()?;
                return Ok(Pattern::Comparison(ComparisonOp::Equal, value));
            }
            #[cfg(feature = "regex")]
            if fork.peek(Token![~]) {
                // =~ operator for pattern matching
                let _: Token![=] = input.parse()?;
                let _: Token![~] = input.parse()?;

                // Performance optimization: if it's a string literal, we can compile
                // the regex at macro expansion time and provide better error messages
                let fork = input.fork();
                if let Ok(lit) = fork.parse::<syn::LitStr>() {
                    // String literal - compile as regex at expansion time for performance
                    // This allows compile-time validation and avoids runtime compilation
                    input.parse::<syn::LitStr>()?;
                    return Ok(Pattern::Regex(lit.value()));
                } else {
                    // Arbitrary expression - use Like trait at runtime
                    let expr = input.parse::<syn::Expr>()?;
                    return Ok(Pattern::Like(expr));
                }
            }
        }
    }

    // Check for slice pattern [...]
    if input.peek(syn::token::Bracket) {
        let content;
        syn::bracketed!(content in input);
        let elements = parse_pattern_list(&content)?;
        return Ok(Pattern::Slice(elements));
    }

    // Check for tuple pattern (...)
    if input.peek(syn::token::Paren) {
        let content;
        syn::parenthesized!(content in input);
        let elements = parse_pattern_list(&content)?;
        return Ok(Pattern::Tuple {
            path: None,
            elements,
        });
    }

    // Try to parse a path (could be Type, Some, Status::Active, etc.)
    let fork = input.fork();
    if let Ok(path) = fork.parse::<syn::Path>() {
        // Check for struct pattern: Path { fields }
        if fork.peek(syn::token::Brace) {
            // Commit to parsing the path
            let path: syn::Path = input.parse()?;
            let content;
            syn::braced!(content in input);
            let expected: Expected = content.parse()?;
            return Ok(Pattern::Struct {
                path,
                fields: expected.fields,
                rest: expected.rest,
            });
        }

        // Check for tuple pattern: Path(...)
        if fork.peek(syn::token::Paren) {
            // Commit to parsing the path
            let path: syn::Path = input.parse()?;
            let content;
            syn::parenthesized!(content in input);

            // Check if the content contains special syntax
            let fork = content.fork();
            let has_special = check_for_special_syntax(&fork);

            if has_special {
                let elements = parse_pattern_list(&content)?;
                return Ok(Pattern::Tuple {
                    path: Some(path),
                    elements,
                });
            } else {
                // Simple expression like Some(vec![1, 2, 3])
                let expr = content.parse()?;
                return Ok(Pattern::Tuple {
                    path: Some(path),
                    elements: vec![Pattern::Simple(expr)],
                });
            }
        }

        // Check if it's a unit variant (path with no brackets)
        // Heuristic: starts with uppercase
        if let Some(segment) = path.segments.last() {
            let name = segment.ident.to_string();
            if name.chars().next().is_some_and(|c| c.is_uppercase()) {
                // Likely a unit variant like None or Status::Inactive
                let path: syn::Path = input.parse()?;
                return Ok(Pattern::Tuple {
                    path: Some(path),
                    elements: vec![],
                });
            }
        }
    }

    // Fall back to simple expression or range
    let expr: syn::Expr = input.parse()?;

    // Check if it's a range expression
    if matches!(expr, syn::Expr::Range(_)) {
        Ok(Pattern::Range(expr))
    } else {
        Ok(Pattern::Simple(expr))
    }
}

// Parse a list of patterns separated by commas
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

// Helper to check if content contains special syntax that needs pattern parsing
fn check_for_special_syntax(content: ParseStream) -> bool {
    // Check for comparison operators
    if content.peek(Token![<]) || content.peek(Token![>]) {
        return true;
    }

    // Check for != operator
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

    // Check for brackets (slice pattern)
    if content.peek(syn::token::Bracket) {
        return true;
    }

    // Check for path followed by braces or parens (struct/enum patterns)
    let fork = content.fork();
    if let Ok(_path) = fork.parse::<syn::Path>() {
        if fork.peek(syn::token::Brace) || fork.peek(syn::token::Paren) {
            return true;
        }
    }

    // Check if there's a comma at the top level - if so, it's multiple elements
    let fork = content.fork();
    if fork.parse::<syn::Expr>().is_ok() && fork.peek(Token![,]) {
        return true; // Multiple elements, needs tuple parsing
    }

    false
}
