use crate::{AssertStruct, ComparisonOp, Expected, FieldAssertion, PatternElement};
use syn::{Result, Token, parse::Parse, parse::ParseStream, punctuated::Punctuated};

pub fn parse(input: proc_macro::TokenStream) -> syn::Result<AssertStruct> {
    syn::parse(input)
}

impl Parse for AssertStruct {
    fn parse(input: ParseStream) -> Result<Self> {
        let value = input.parse()?;
        let _: Token![,] = input.parse()?;
        let type_name = input.parse()?;
        let content;
        syn::braced!(content in input);
        let expected = content.parse()?;

        Ok(AssertStruct {
            value,
            type_name,
            expected,
        })
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

impl Parse for FieldAssertion {
    fn parse(input: ParseStream) -> Result<Self> {
        let field_name: syn::Ident = input.parse()?;
        let _: Token![:] = input.parse()?;

        // Check for comparison operators at the top level: <, <=, >, >=, ==, !=
        if input.peek(Token![<]) {
            let _: Token![<] = input.parse()?;
            if input.peek(Token![=]) {
                let _: Token![=] = input.parse()?;
                let value = input.parse()?;
                return Ok(FieldAssertion::Comparison {
                    field_name,
                    op: ComparisonOp::LessEqual,
                    value,
                });
            } else {
                let value = input.parse()?;
                return Ok(FieldAssertion::Comparison {
                    field_name,
                    op: ComparisonOp::Less,
                    value,
                });
            }
        }

        if input.peek(Token![>]) {
            let _: Token![>] = input.parse()?;
            if input.peek(Token![=]) {
                let _: Token![=] = input.parse()?;
                let value = input.parse()?;
                return Ok(FieldAssertion::Comparison {
                    field_name,
                    op: ComparisonOp::GreaterEqual,
                    value,
                });
            } else {
                let value = input.parse()?;
                return Ok(FieldAssertion::Comparison {
                    field_name,
                    op: ComparisonOp::Greater,
                    value,
                });
            }
        }

        // Check for != operator
        if input.peek(Token![!]) {
            let fork = input.fork();
            if fork.parse::<Token![!]>().is_ok() && fork.peek(Token![=]) {
                let _: Token![!] = input.parse()?;
                let _: Token![=] = input.parse()?;
                let value = input.parse()?;
                return Ok(FieldAssertion::Comparison {
                    field_name,
                    op: ComparisonOp::NotEqual,
                    value,
                });
            }
        }

        // Check for == or =~ operators
        if input.peek(Token![=]) {
            let fork = input.fork();
            if fork.parse::<Token![=]>().is_ok() {
                if fork.peek(Token![=]) {
                    // This is the == operator
                    let _: Token![=] = input.parse()?;
                    let _: Token![=] = input.parse()?;
                    let value = input.parse()?;
                    return Ok(FieldAssertion::Comparison {
                        field_name,
                        op: ComparisonOp::Equal,
                        value,
                    });
                }
                #[cfg(feature = "regex")]
                if fork.peek(Token![~]) {
                    // This is the =~ regex operator
                    let _: Token![=] = input.parse()?;
                    let _: Token![~] = input.parse()?;

                    // Expect a string literal (raw or regular)
                    let lit: syn::LitStr = input.parse()?;
                    return Ok(FieldAssertion::Regex {
                        field_name,
                        pattern: lit.value(),
                    });
                }
            }
        }

        // Check if this is a slice pattern [...]
        if input.peek(syn::token::Bracket) {
            let content;
            syn::bracketed!(content in input);

            let elements = parse_slice_elements(&content)?;

            return Ok(FieldAssertion::SlicePattern {
                field_name,
                elements,
            });
        }

        // Check if this is a plain tuple (no path prefix)
        if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);

            let elements = parse_tuple_elements(&content)?;

            return Ok(FieldAssertion::TuplePattern {
                field_name,
                path: None, // No path for plain tuples
                elements,
            });
        }

        // Try to parse a path (could be Type, Some, Status::Active, etc.)
        let fork = input.fork();
        if let Ok(path) = fork.parse::<syn::Path>() {
            // We have a path, check what follows

            // Check for struct pattern: Path { fields }
            if fork.peek(syn::token::Brace) {
                // Commit to parsing the path
                let path: syn::Path = input.parse()?;
                let content;
                syn::braced!(content in input);
                let nested = content.parse()?;

                return Ok(FieldAssertion::StructPattern {
                    field_name,
                    path,
                    nested,
                });
            }

            // Check for tuple pattern: Path(...)
            if fork.peek(syn::token::Paren) {
                // Commit to parsing the path
                let path: syn::Path = input.parse()?;

                // Now we need to check if the parentheses contain special syntax
                // We'll parse the parenthesized content manually
                let elements = parse_variant_tuple_contents(input)?;

                return Ok(FieldAssertion::TuplePattern {
                    field_name,
                    path: Some(path),
                    elements,
                });
            }

            // Check if it's a unit variant (path with no brackets)
            // We need to distinguish between unit variants and simple values
            // For now, we'll check if it starts with uppercase (convention)
            if let Some(segment) = path.segments.last() {
                let name = segment.ident.to_string();
                if name.chars().next().is_some_and(|c| c.is_uppercase()) {
                    // Likely a unit variant like None or Status::Inactive
                    let path: syn::Path = input.parse()?;
                    return Ok(FieldAssertion::UnitPattern { field_name, path });
                }
            }
        }

        // Fall back to simple field assertion - but check if it's a range first
        let expected_value: syn::Expr = input.parse()?;

        // Check if it's a range expression
        if matches!(expected_value, syn::Expr::Range(_)) {
            return Ok(FieldAssertion::Range {
                field_name,
                range: expected_value,
            });
        }

        Ok(FieldAssertion::Simple {
            field_name,
            expected_value,
        })
    }
}

// Parse the contents of a variant's tuple - handles special syntax like Some(> 30) or Foo(> 1, < 10)
fn parse_variant_tuple_contents(input: ParseStream) -> Result<Vec<PatternElement>> {
    // We need to look at the raw tokens to determine if there's special syntax
    let content;
    let _paren = syn::parenthesized!(content in input);

    // Check if the content contains special syntax that requires special parsing
    // Otherwise, treat it as a simple expression
    let fork = content.fork();
    let has_special_syntax = check_for_special_syntax(&fork);

    if has_special_syntax {
        // Parse all elements using the same logic as plain tuples
        // This now supports comparison operators and regex in any position
        parse_tuple_elements(&content)
    } else {
        // Parse as a single simple expression (could be a tuple literal, vec![], etc.)
        let expr = content.parse()?;
        Ok(vec![PatternElement::Simple(expr)])
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

// Parse the contents of a slice pattern [...]
fn parse_slice_elements(content: ParseStream) -> Result<Vec<PatternElement>> {
    // Similar to parse_tuple_elements but for slice patterns
    // This allows comparison operators, regex, and nested patterns in slices
    parse_pattern_elements(content)
}

// Parse the contents of a tuple pattern (for plain tuples without a variant)
fn parse_tuple_elements(content: ParseStream) -> Result<Vec<PatternElement>> {
    parse_pattern_elements(content)
}

// Common pattern element parsing used by both slices and tuples
fn parse_pattern_elements(content: ParseStream) -> Result<Vec<PatternElement>> {
    let mut elements = Vec::new();

    while !content.is_empty() {
        // Check for .. (rest pattern)
        if content.peek(Token![..]) {
            let _: Token![..] = content.parse()?;
            elements.push(PatternElement::Rest);
        }
        // Check for nested parentheses (nested tuple)
        else if content.peek(syn::token::Paren) {
            // It's a nested tuple like ((10, 20), ...)
            let inner_content;
            syn::parenthesized!(inner_content in content);
            let inner_elements = parse_pattern_elements(&inner_content)?;
            // Wrap in a TuplePattern without a path
            elements.push(PatternElement::Tuple(None, inner_elements));
        }
        // Check for comparison operators
        else if content.peek(Token![<]) || content.peek(Token![>]) {
            // It's a comparison pattern
            let op = if content.peek(Token![<]) {
                let _: Token![<] = content.parse()?;
                if content.peek(Token![=]) {
                    let _: Token![=] = content.parse()?;
                    ComparisonOp::LessEqual
                } else {
                    ComparisonOp::Less
                }
            } else {
                let _: Token![>] = content.parse()?;
                if content.peek(Token![=]) {
                    let _: Token![=] = content.parse()?;
                    ComparisonOp::GreaterEqual
                } else {
                    ComparisonOp::Greater
                }
            };

            let value = content.parse()?;
            elements.push(PatternElement::Comparison(op, value));
        }
        // Check for != operator
        else if content.peek(Token![!]) {
            let fork = content.fork();
            if fork.parse::<Token![!]>().is_ok() && fork.peek(Token![=]) {
                let _: Token![!] = content.parse()?;
                let _: Token![=] = content.parse()?;
                let value = content.parse()?;
                elements.push(PatternElement::Comparison(ComparisonOp::NotEqual, value));
            } else {
                // Not a != operator, parse as regular expression
                let expr = content.parse()?;
                elements.push(PatternElement::Simple(expr));
            }
        }
        // Check for == or =~ operators
        else if content.peek(Token![=]) {
            let fork = content.fork();
            if fork.parse::<Token![=]>().is_ok() {
                if fork.peek(Token![=]) {
                    // == operator
                    let _: Token![=] = content.parse()?;
                    let _: Token![=] = content.parse()?;
                    let value = content.parse()?;
                    elements.push(PatternElement::Comparison(ComparisonOp::Equal, value));
                } else if fork.peek(Token![~]) {
                    // =~ regex operator
                    #[cfg(feature = "regex")]
                    {
                        let _: Token![=] = content.parse()?;
                        let _: Token![~] = content.parse()?;
                        let lit: syn::LitStr = content.parse()?;
                        elements.push(PatternElement::Regex(lit.value()));
                    }
                    #[cfg(not(feature = "regex"))]
                    {
                        // If regex feature is disabled, parse as regular expression
                        let expr = content.parse()?;
                        elements.push(PatternElement::Simple(expr));
                    }
                } else {
                    // Just a single =, parse as regular expression
                    let expr = content.parse()?;
                    elements.push(PatternElement::Simple(expr));
                }
            } else {
                // Not an operator, parse as regular expression
                let expr = content.parse()?;
                elements.push(PatternElement::Simple(expr));
            }
        }
        // Try to parse as path (could be struct pattern or enum variant)
        else {
            let fork = content.fork();
            if let Ok(path) = fork.parse::<syn::Path>() {
                // Check if it's followed by braces (struct pattern)
                if fork.peek(syn::token::Brace) {
                    // It's a nested struct pattern
                    let inner_path: syn::Path = content.parse()?;
                    let inner_content;
                    syn::braced!(inner_content in content);
                    let inner_nested = inner_content.parse()?;

                    elements.push(PatternElement::Struct(inner_path, inner_nested));
                }
                // Check if it's followed by parentheses (enum tuple variant like Some(...))
                else if fork.peek(syn::token::Paren) {
                    // It's an enum variant with tuple data
                    let variant_path: syn::Path = content.parse()?;
                    let variant_elements = parse_variant_tuple_contents(content)?;
                    elements.push(PatternElement::Tuple(Some(variant_path), variant_elements));
                } else {
                    // It's either a unit variant or regular expression
                    // Try to determine which by checking if it starts with uppercase
                    if let Some(segment) = path.segments.last() {
                        let name = segment.ident.to_string();
                        if name.chars().next().is_some_and(|c| c.is_uppercase()) {
                            // Likely a unit variant like None
                            let variant_path: syn::Path = content.parse()?;
                            elements.push(PatternElement::Tuple(Some(variant_path), vec![]));
                        } else {
                            // Regular expression
                            let expr = content.parse()?;
                            elements.push(PatternElement::Simple(expr));
                        }
                    } else {
                        // Regular expression
                        let expr = content.parse()?;
                        elements.push(PatternElement::Simple(expr));
                    }
                }
            } else {
                // Regular expression
                let expr = content.parse()?;
                elements.push(PatternElement::Simple(expr));
            }
        }

        // Handle comma
        if !content.is_empty() {
            let _: Token![,] = content.parse()?;
        }
    }

    Ok(elements)
}
