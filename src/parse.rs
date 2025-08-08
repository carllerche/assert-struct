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

        // Check for comparison operators at the top level: <, <=, >, >=
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

        // Check for =~ regex operator at the top level
        #[cfg(feature = "regex")]
        if input.peek(Token![=]) {
            let fork = input.fork();
            if fork.parse::<Token![=]>().is_ok() && fork.peek(Token![~]) {
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

        // Fall back to simple field assertion
        let expected_value = input.parse()?;
        Ok(FieldAssertion::Simple {
            field_name,
            expected_value,
        })
    }
}

// Parse the contents of a variant's tuple - handles special syntax like Some(> 30)
fn parse_variant_tuple_contents(input: ParseStream) -> Result<Vec<PatternElement>> {
    // We need to look at the raw tokens to determine if there's special syntax
    let content;
    let _paren = syn::parenthesized!(content in input);

    // Peek at the first token to see if it's a special pattern
    if content.peek(Token![<]) || content.peek(Token![>]) {
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
        return Ok(vec![PatternElement::Comparison(op, value)]);
    }

    // Check for regex pattern
    #[cfg(feature = "regex")]
    if content.peek(Token![=]) {
        // Might be a regex pattern
        let fork = content.fork();
        if fork.parse::<Token![=]>().is_ok() && fork.peek(Token![~]) {
            let _: Token![=] = content.parse()?;
            let _: Token![~] = content.parse()?;
            let lit: syn::LitStr = content.parse()?;
            return Ok(vec![PatternElement::Regex(lit.value())]);
        }
    }

    // Check for nested struct pattern
    let fork = content.fork();
    if let Ok(_inner_path) = fork.parse::<syn::Path>() {
        if fork.peek(syn::token::Brace) {
            // It's a nested struct pattern inside the tuple
            let inner_path: syn::Path = content.parse()?;
            let inner_content;
            syn::braced!(inner_content in content);
            let inner_nested = inner_content.parse()?;

            return Ok(vec![PatternElement::Struct(inner_path, inner_nested)]);
        }
    }

    // Parse as normal tuple elements
    parse_tuple_elements(&content)
}

// Parse the contents of a tuple pattern (for plain tuples without a variant)
fn parse_tuple_elements(content: ParseStream) -> Result<Vec<PatternElement>> {
    let mut elements = Vec::new();

    while !content.is_empty() {
        // For plain tuples, we don't support special syntax at the top level
        // (since (> 30) wouldn't be valid Rust)
        // So we just parse expressions

        // Try to parse as a nested struct pattern first
        let fork = content.fork();
        if let Ok(_inner_path) = fork.parse::<syn::Path>() {
            if fork.peek(syn::token::Brace) {
                // It's a nested struct pattern
                let inner_path: syn::Path = content.parse()?;
                let inner_content;
                syn::braced!(inner_content in content);
                let inner_nested = inner_content.parse()?;

                elements.push(PatternElement::Struct(inner_path, inner_nested));
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

        // Handle comma
        if !content.is_empty() {
            let _: Token![,] = content.parse()?;
        }
    }

    Ok(elements)
}
