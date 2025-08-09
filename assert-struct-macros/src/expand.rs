use crate::{AssertStruct, ComparisonOp, FieldAssertion, Pattern};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Token, punctuated::Punctuated};

pub fn expand(assert: &AssertStruct) -> TokenStream {
    let value = &assert.value;
    let pattern = &assert.pattern;

    // Generate the assertion for the root pattern
    let assertion = generate_pattern_assertion(&quote! { #value }, pattern, false);

    quote! {
        {
            #assertion
        }
    }
}

// Generate assertion code for any pattern
// is_ref indicates whether value_expr is already a reference
fn generate_pattern_assertion(
    value_expr: &TokenStream,
    pattern: &Pattern,
    is_ref: bool,
) -> TokenStream {
    match pattern {
        Pattern::Simple(expected) => {
            // Simple value comparison
            let transformed = transform_expected_value(expected);
            if is_ref {
                quote! {
                    assert_eq!(#value_expr, &#transformed);
                }
            } else {
                quote! {
                    assert_eq!(&#value_expr, &#transformed);
                }
            }
        }
        Pattern::Struct { path, fields, rest } => {
            // Check if this is an enum variant (path has multiple segments or starts with uppercase)
            let is_enum_variant = if path.segments.len() > 1 {
                true
            } else if let Some(segment) = path.segments.first() {
                segment
                    .ident
                    .to_string()
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_uppercase())
            } else {
                false
            };

            if is_enum_variant {
                // Enum struct variant - generate match expression
                generate_enum_struct_assertion(value_expr, path, fields, *rest, is_ref)
            } else {
                // Regular struct - use let destructuring
                let field_names: Vec<_> = fields.iter().map(|f| &f.field_name).collect();

                let rest_pattern = if *rest {
                    quote! { , .. }
                } else {
                    quote! {}
                };

                let field_assertions: Vec<_> = fields
                    .iter()
                    .map(|f| {
                        let field_name = &f.field_name;
                        let field_pattern = &f.pattern;
                        // Fields from destructuring are references
                        generate_pattern_assertion(&quote! { #field_name }, field_pattern, true)
                    })
                    .collect();

                quote! {
                    let #path { #(#field_names),* #rest_pattern } = &#value_expr;
                    #(#field_assertions)*
                }
            }
        }
        Pattern::Tuple { path, elements } => {
            // Handle both plain tuples and enum variants
            if let Some(variant_path) = path {
                // Enum variant (Some(...), None, Ok(...), etc.)
                if elements.is_empty() {
                    // Unit variant like None
                    generate_unit_variant_assertion(value_expr, variant_path, is_ref)
                } else {
                    // Tuple variant with data
                    generate_enum_tuple_assertion(value_expr, variant_path, elements, is_ref)
                }
            } else {
                // Plain tuple
                generate_plain_tuple_assertion(value_expr, elements, is_ref)
            }
        }
        Pattern::Slice(elements) => {
            // Slice pattern using Rust's native slice matching
            generate_slice_assertion(value_expr, elements, is_ref)
        }
        Pattern::Comparison(op, value) => {
            // Comparison operators
            generate_comparison_assertion(value_expr, op, value, is_ref)
        }
        Pattern::Range(range) => {
            // Range pattern
            if is_ref {
                quote! {
                    match #value_expr {
                        #range => {},
                        _ => panic!(
                            "Value not in range: {:?} not matching pattern",
                            #value_expr
                        ),
                    }
                }
            } else {
                quote! {
                    match &#value_expr {
                        #range => {},
                        _ => panic!(
                            "Value not in range: {:?} not matching pattern",
                            &#value_expr
                        ),
                    }
                }
            }
        }
        #[cfg(feature = "regex")]
        Pattern::Regex(pattern_str) => {
            // Regex pattern
            if is_ref {
                quote! {
                    {
                        let re = ::regex::Regex::new(#pattern_str)
                            .expect(concat!("Invalid regex pattern: ", #pattern_str));
                        if !re.is_match(#value_expr) {
                            panic!(
                                "Value does not match regex pattern `{}`\n  value: {:?}",
                                #pattern_str,
                                #value_expr
                            );
                        }
                    }
                }
            } else {
                quote! {
                    {
                        let re = ::regex::Regex::new(#pattern_str)
                            .expect(concat!("Invalid regex pattern: ", #pattern_str));
                        if !re.is_match(&#value_expr) {
                            panic!(
                                "Value does not match regex pattern `{}`\n  value: {:?}",
                                #pattern_str,
                                &#value_expr
                            );
                        }
                    }
                }
            }
        }
        Pattern::Rest => {
            // Rest patterns don't generate assertions themselves
            quote! {}
        }
    }
}

// Generate assertion for unit variants like None
fn generate_unit_variant_assertion(
    value_expr: &TokenStream,
    path: &syn::Path,
    is_ref: bool,
) -> TokenStream {
    // Special handling for None
    if is_option_none_path(path) {
        if is_ref {
            quote! {
                match #value_expr {
                    None => {},
                    Some(_) => panic!("Expected None, got Some"),
                }
            }
        } else {
            quote! {
                match &#value_expr {
                    None => {},
                    Some(_) => panic!("Expected None, got Some"),
                }
            }
        }
    } else {
        // General unit variant
        if is_ref {
            quote! {
                match #value_expr {
                    #path => {},
                    _ => panic!(
                        "Expected {}, got {:?}",
                        stringify!(#path),
                        #value_expr
                    ),
                }
            }
        } else {
            quote! {
                match &#value_expr {
                    #path => {},
                    _ => panic!(
                        "Expected {}, got {:?}",
                        stringify!(#path),
                        &#value_expr
                    ),
                }
            }
        }
    }
}

// Generate assertion for enum struct variants
fn generate_enum_struct_assertion(
    value_expr: &TokenStream,
    path: &syn::Path,
    fields: &Punctuated<FieldAssertion, Token![,]>,
    rest: bool,
    is_ref: bool,
) -> TokenStream {
    let field_names: Vec<_> = fields.iter().map(|f| &f.field_name).collect();

    let rest_pattern = if rest {
        quote! { , .. }
    } else {
        quote! {}
    };

    let field_assertions: Vec<_> = fields
        .iter()
        .map(|f| {
            let field_name = &f.field_name;
            let field_pattern = &f.pattern;
            // Fields from destructuring are references
            generate_pattern_assertion(&quote! { #field_name }, field_pattern, true)
        })
        .collect();

    if is_ref {
        quote! {
            match #value_expr {
                #path { #(#field_names),* #rest_pattern } => {
                    #(#field_assertions)*
                },
                _ => panic!(
                    "Expected {}, got {:?}",
                    stringify!(#path),
                    #value_expr
                ),
            }
        }
    } else {
        quote! {
            match &#value_expr {
                #path { #(#field_names),* #rest_pattern } => {
                    #(#field_assertions)*
                },
                _ => panic!(
                    "Expected {}, got {:?}",
                    stringify!(#path),
                    &#value_expr
                ),
            }
        }
    }
}

// Generate assertion for enum tuple variants
fn generate_enum_tuple_assertion(
    value_expr: &TokenStream,
    path: &syn::Path,
    elements: &[Pattern],
    is_ref: bool,
) -> TokenStream {
    // Special handling for Some
    if is_option_some_path(path) && elements.len() == 1 {
        let inner_assertion = generate_pattern_assertion(
            &quote! { inner },
            &elements[0],
            true, // inner is a reference from the match
        );
        if is_ref {
            return quote! {
                match #value_expr {
                    Some(inner) => {
                        #inner_assertion
                    },
                    None => panic!("Expected Some(...), got None"),
                }
            };
        } else {
            return quote! {
                match &#value_expr {
                    Some(inner) => {
                        #inner_assertion
                    },
                    None => panic!("Expected Some(...), got None"),
                }
            };
        }
    }

    // General enum tuple variant
    let element_names: Vec<_> = (0..elements.len())
        .map(|i| quote::format_ident!("__elem_{}", i))
        .collect();

    let element_assertions: Vec<_> = element_names
        .iter()
        .zip(elements)
        .map(|(name, pattern)| generate_pattern_assertion(&quote! { #name }, pattern, true))
        .collect();

    if is_ref {
        quote! {
            match #value_expr {
                #path(#(#element_names),*) => {
                    #(#element_assertions)*
                },
                _ => panic!(
                    "Expected {}, got {:?}",
                    stringify!(#path),
                    #value_expr
                ),
            }
        }
    } else {
        quote! {
            match &#value_expr {
                #path(#(#element_names),*) => {
                    #(#element_assertions)*
                },
                _ => panic!(
                    "Expected {}, got {:?}",
                    stringify!(#path),
                    &#value_expr
                ),
            }
        }
    }
}

// Generate assertion for plain tuples
fn generate_plain_tuple_assertion(
    value_expr: &TokenStream,
    elements: &[Pattern],
    is_ref: bool,
) -> TokenStream {
    let element_names: Vec<_> = (0..elements.len())
        .map(|i| quote::format_ident!("__tuple_elem_{}", i))
        .collect();

    let destructure = if is_ref {
        quote! {
            let (#(#element_names),*) = #value_expr;
        }
    } else {
        quote! {
            let (#(#element_names),*) = &#value_expr;
        }
    };

    let element_assertions: Vec<_> = element_names
        .iter()
        .zip(elements)
        .map(|(name, pattern)| generate_pattern_assertion(&quote! { #name }, pattern, true))
        .collect();

    quote! {
        {
            #destructure
            #(#element_assertions)*
        }
    }
}

// Generate assertion for slice patterns
fn generate_slice_assertion(
    value_expr: &TokenStream,
    elements: &[Pattern],
    _is_ref: bool,
) -> TokenStream {
    let mut pattern_parts = Vec::new();
    let mut bindings_and_assertions = Vec::new();

    for (i, elem) in elements.iter().enumerate() {
        match elem {
            Pattern::Rest => {
                pattern_parts.push(quote! { .. });
            }
            _ => {
                let binding = quote::format_ident!("__elem_{}", i);
                pattern_parts.push(quote! { #binding });

                let assertion = generate_pattern_assertion(&quote! { #binding }, elem, true);
                bindings_and_assertions.push(assertion);
            }
        }
    }

    let slice_expr = quote! { (#value_expr).as_slice() };

    quote! {
        match #slice_expr {
            [#(#pattern_parts),*] => {
                #(#bindings_and_assertions)*
            }
            _ => panic!(
                "Pattern mismatch: {:?} doesn't match expected pattern",
                &#value_expr
            ),
        }
    }
}

// Generate comparison assertion
fn generate_comparison_assertion(
    value_expr: &TokenStream,
    op: &ComparisonOp,
    expected: &Expr,
    is_ref: bool,
) -> TokenStream {
    let (op_str, error_msg) = match op {
        ComparisonOp::Less => ("<", "comparison"),
        ComparisonOp::LessEqual => ("<=", "comparison"),
        ComparisonOp::Greater => (">", "comparison"),
        ComparisonOp::GreaterEqual => (">=", "comparison"),
        ComparisonOp::Equal => ("==", "equality"),
        ComparisonOp::NotEqual => ("!=", "inequality"),
    };

    if is_ref {
        let comparison = match op {
            ComparisonOp::Less => quote! { #value_expr < &(#expected) },
            ComparisonOp::LessEqual => quote! { #value_expr <= &(#expected) },
            ComparisonOp::Greater => quote! { #value_expr > &(#expected) },
            ComparisonOp::GreaterEqual => quote! { #value_expr >= &(#expected) },
            ComparisonOp::Equal => quote! { #value_expr == &(#expected) },
            ComparisonOp::NotEqual => quote! { #value_expr != &(#expected) },
        };

        quote! {
            if !(#comparison) {
                panic!(
                    "Failed {}: {:?} {} {:?}",
                    #error_msg,
                    #value_expr,
                    #op_str,
                    &(#expected)
                );
            }
        }
    } else {
        let comparison = match op {
            ComparisonOp::Less => quote! { &#value_expr < &(#expected) },
            ComparisonOp::LessEqual => quote! { &#value_expr <= &(#expected) },
            ComparisonOp::Greater => quote! { &#value_expr > &(#expected) },
            ComparisonOp::GreaterEqual => quote! { &#value_expr >= &(#expected) },
            ComparisonOp::Equal => quote! { &#value_expr == &(#expected) },
            ComparisonOp::NotEqual => quote! { &#value_expr != &(#expected) },
        };

        quote! {
            if !(#comparison) {
                panic!(
                    "Failed {}: {:?} {} {:?}",
                    #error_msg,
                    &#value_expr,
                    #op_str,
                    &(#expected)
                );
            }
        }
    }
}

// Transform expected values (e.g., string literals to String)
fn transform_expected_value(expr: &Expr) -> Expr {
    match expr {
        Expr::Lit(lit) if matches!(lit.lit, syn::Lit::Str(_)) => {
            // Transform "literal" to "literal".to_string() for String fields
            syn::parse_quote! { #expr.to_string() }
        }
        _ => expr.clone(),
    }
}

// Check if a path refers to Option::Some
fn is_option_some_path(path: &syn::Path) -> bool {
    if let Some(segment) = path.segments.last() {
        segment.ident == "Some"
    } else {
        false
    }
}

// Check if a path refers to Option::None
fn is_option_none_path(path: &syn::Path) -> bool {
    if let Some(segment) = path.segments.last() {
        segment.ident == "None"
    } else {
        false
    }
}
