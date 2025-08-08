use crate::{AssertStruct, ComparisonOp, Expected, FieldAssertion};
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

        // Check for comparison operators: <, <=, >, >=
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

        // Check for =~ regex operator
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

        // Check if this is a tuple by looking for parentheses
        if input.peek(syn::token::Paren) {
            // This is a tuple
            let content;
            syn::parenthesized!(content in input);
            let mut elements = Vec::new();

            // Parse comma-separated elements
            while !content.is_empty() {
                elements.push(content.parse()?);
                if !content.is_empty() {
                    let _: Token![,] = content.parse()?;
                }
            }

            Ok(FieldAssertion::Tuple {
                field_name,
                elements,
            })
        } else {
            // Check if this is a nested struct by looking for a type name followed by braces
            let fork = input.fork();
            if fork.parse::<syn::Path>().is_ok() && fork.peek(syn::token::Brace) {
                // This is a nested struct
                let type_name: syn::Path = input.parse()?;
                let content;
                syn::braced!(content in input);
                let nested = content.parse()?;
                Ok(FieldAssertion::Nested {
                    field_name,
                    type_name,
                    nested,
                })
            } else {
                // Simple field assertion
                let expected_value = input.parse()?;
                Ok(FieldAssertion::Simple {
                    field_name,
                    expected_value,
                })
            }
        }
    }
}
