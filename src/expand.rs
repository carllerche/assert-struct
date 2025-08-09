use crate::{AssertStruct, ComparisonOp, Expected, FieldAssertion, PatternElement};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

pub fn expand(assert: &AssertStruct) -> TokenStream {
    let value = &assert.value;
    let type_name = &assert.type_name;

    // Collect all field names for destructuring
    let field_names: Vec<_> = assert
        .expected
        .fields
        .iter()
        .map(|f| match f {
            FieldAssertion::Simple { field_name, .. } => field_name.clone(),
            FieldAssertion::StructPattern { field_name, .. } => field_name.clone(),
            FieldAssertion::TuplePattern { field_name, .. } => field_name.clone(),
            FieldAssertion::UnitPattern { field_name, .. } => field_name.clone(),
            #[cfg(feature = "regex")]
            FieldAssertion::Regex { field_name, .. } => field_name.clone(),
            FieldAssertion::Comparison { field_name, .. } => field_name.clone(),
        })
        .collect();

    // Handle partial matching with ..
    let rest_pattern = if assert.expected.rest {
        quote! { , .. }
    } else {
        quote! {}
    };

    // Generate the destructuring pattern with references to avoid moves
    // In Rust 2024 edition, we use & on the value instead of ref in the pattern
    let destructure = quote! {
        let #type_name { #(#field_names),* #rest_pattern } = &#value;
    };

    // Generate assertions for each field
    let assertions = generate_assertions(&assert.expected);

    quote! {
        {
            #destructure
            #assertions
        }
    }
}

fn generate_assertions(expected: &Expected) -> TokenStream {
    let mut assertions = Vec::new();

    for field in &expected.fields {
        match field {
            FieldAssertion::Simple {
                field_name,
                expected_value,
                ..
            } => {
                // Simple field: generate assert_eq! with reference comparison
                assertions.push(quote! {
                    assert_eq!(#field_name, &#expected_value);
                });
            }
            FieldAssertion::StructPattern {
                field_name,
                path,
                nested,
                ..
            } => {
                // Struct pattern: recursively generate assertions
                let assertion = generate_struct_pattern_assert(field_name, path, nested);
                assertions.push(assertion);
            }
            FieldAssertion::TuplePattern {
                field_name,
                path,
                elements,
                ..
            } => {
                // Tuple pattern: generate match expression
                let assertion = generate_tuple_pattern_assert(field_name, path, elements);
                assertions.push(assertion);
            }
            FieldAssertion::UnitPattern {
                field_name, path, ..
            } => {
                // Unit pattern: check if field matches the variant
                let assertion = generate_unit_pattern_assert(field_name, path);
                assertions.push(assertion);
            }
            #[cfg(feature = "regex")]
            FieldAssertion::Regex {
                field_name,
                pattern,
                ..
            } => {
                // Regex: compile pattern and check if field matches
                assertions.push(quote! {
                    {
                        let re = ::regex::Regex::new(#pattern)
                            .expect(concat!("Invalid regex pattern: ", #pattern));
                        if !re.is_match(#field_name) {
                            panic!(
                                "Field `{}` does not match regex pattern `{}`\n  value: {:?}",
                                stringify!(#field_name),
                                #pattern,
                                #field_name
                            );
                        }
                    }
                });
            }
            FieldAssertion::Comparison {
                field_name,
                op,
                value,
                ..
            } => {
                // Comparison: generate appropriate comparison assertion
                let op_str = match op {
                    ComparisonOp::Less => "<",
                    ComparisonOp::LessEqual => "<=",
                    ComparisonOp::Greater => ">",
                    ComparisonOp::GreaterEqual => ">=",
                };

                let comparison = match op {
                    ComparisonOp::Less => quote! { #field_name < &#value },
                    ComparisonOp::LessEqual => quote! { #field_name <= &#value },
                    ComparisonOp::Greater => quote! { #field_name > &#value },
                    ComparisonOp::GreaterEqual => quote! { #field_name >= &#value },
                };

                assertions.push(quote! {
                    if !(#comparison) {
                        panic!(
                            "Field `{}` failed comparison: {:?} {} {}",
                            stringify!(#field_name),
                            #field_name,
                            #op_str,
                            &#value
                        );
                    }
                });
            }
        }
    }

    quote! {
        #(#assertions)*
    }
}

/// Generate assertion for struct patterns (both standalone and enum variants)
fn generate_struct_pattern_assert(
    field_name: &syn::Ident,
    path: &syn::Path,
    nested: &Expected,
) -> TokenStream {
    // Check if this looks like an enum variant
    // Heuristic: if the path has multiple segments (e.g., Status::Active), it's likely an enum variant
    // Single segment paths are likely regular structs (even though they start with uppercase)
    let is_enum_variant = path.segments.len() > 1;

    if is_enum_variant {
        // It's an enum variant with struct data
        // Generate: match field { Path { fields.. } => check, _ => panic }

        // Collect field names for destructuring
        let field_names: Vec<_> = nested
            .fields
            .iter()
            .map(|f| match f {
                FieldAssertion::Simple { field_name, .. } => field_name.clone(),
                FieldAssertion::StructPattern { field_name, .. } => field_name.clone(),
                FieldAssertion::TuplePattern { field_name, .. } => field_name.clone(),
                FieldAssertion::UnitPattern { field_name, .. } => field_name.clone(),
                #[cfg(feature = "regex")]
                FieldAssertion::Regex { field_name, .. } => field_name.clone(),
                FieldAssertion::Comparison { field_name, .. } => field_name.clone(),
            })
            .collect();

        let rest_pattern = if nested.rest {
            quote! { , .. }
        } else {
            quote! {}
        };

        let nested_assertions = generate_assertions(nested);

        quote! {
            match #field_name {
                #path { #(#field_names),* #rest_pattern } => {
                    #nested_assertions
                },
                _ => panic!(
                    "Field `{}` expected {}, got {:?}",
                    stringify!(#field_name),
                    stringify!(#path),
                    #field_name
                ),
            }
        }
    } else {
        // It's a regular struct, use recursive assert_struct!
        // Generate the field assignments for the expected struct
        let field_assignments = generate_struct_field_assignments(nested);

        let rest_pattern = if nested.rest {
            quote! { , .. }
        } else {
            quote! {}
        };

        quote! {
            assert_struct!(#field_name, #path {
                #(#field_assignments),*
                #rest_pattern
            });
        }
    }
}

/// Generate assertion for tuple patterns (both standalone and enum variants)
fn generate_tuple_pattern_assert(
    field_name: &syn::Ident,
    path: &Option<syn::Path>,
    elements: &[PatternElement],
) -> TokenStream {
    if let Some(variant_path) = path {
        // It's an enum variant with tuple data
        // Check for special cases first

        // Special handling for Some/None patterns
        if is_option_some_path(variant_path) && elements.len() == 1 {
            return generate_option_assertion(field_name, &elements[0]);
        }

        // General enum tuple variant
        let element_names: Vec<_> = (0..elements.len())
            .map(|i| quote::format_ident!("__elem_{}", i))
            .collect();

        let element_assertions = elements
            .iter()
            .zip(&element_names)
            .map(|(elem, name)| generate_pattern_element_assertion(name, elem))
            .collect::<Vec<_>>();

        quote! {
            match #field_name {
                #variant_path(#(#element_names),*) => {
                    #(#element_assertions)*
                },
                _ => panic!(
                    "Field `{}` expected {}, got {:?}",
                    stringify!(#field_name),
                    stringify!(#variant_path),
                    #field_name
                ),
            }
        }
    } else {
        // Plain tuple
        let element_names: Vec<_> = (0..elements.len())
            .map(|i| quote::format_ident!("__tuple_elem_{}", i))
            .collect();

        let destructure = quote! {
            let (#(#element_names),*) = &#field_name;
        };

        let element_assertions = elements
            .iter()
            .zip(&element_names)
            .map(|(elem, name)| generate_pattern_element_assertion(name, elem))
            .collect::<Vec<_>>();

        quote! {
            {
                #destructure
                #(#element_assertions)*
            }
        }
    }
}

/// Generate assertion for a single pattern element
fn generate_pattern_element_assertion(
    elem_name: &syn::Ident,
    element: &PatternElement,
) -> TokenStream {
    match element {
        PatternElement::Simple(expected) => {
            // Check if this is a string literal that needs transformation
            let transformed = transform_expected_value(expected);
            quote! {
                assert_eq!(#elem_name, &#transformed);
            }
        }
        PatternElement::Comparison(op, value) => {
            let op_str = match op {
                ComparisonOp::Less => "<",
                ComparisonOp::LessEqual => "<=",
                ComparisonOp::Greater => ">",
                ComparisonOp::GreaterEqual => ">=",
            };

            let comparison = match op {
                ComparisonOp::Less => quote! { #elem_name < &#value },
                ComparisonOp::LessEqual => quote! { #elem_name <= &#value },
                ComparisonOp::Greater => quote! { #elem_name > &#value },
                ComparisonOp::GreaterEqual => quote! { #elem_name >= &#value },
            };

            quote! {
                if !(#comparison) {
                    panic!(
                        "Element failed comparison: {:?} {} {}",
                        #elem_name,
                        #op_str,
                        &#value
                    );
                }
            }
        }
        #[cfg(feature = "regex")]
        PatternElement::Regex(pattern) => {
            quote! {
                {
                    let re = ::regex::Regex::new(#pattern)
                        .expect(concat!("Invalid regex pattern: ", #pattern));
                    if !re.is_match(#elem_name) {
                        panic!(
                            "Element does not match regex pattern `{}`\n  value: {:?}",
                            #pattern,
                            #elem_name
                        );
                    }
                }
            }
        }
        PatternElement::Struct(struct_path, nested) => {
            // Check if this is an enum variant or a regular struct
            let is_enum_variant = struct_path.segments.len() > 1;

            if is_enum_variant {
                // It's an enum variant - generate a match expression
                let field_names: Vec<_> = nested
                    .fields
                    .iter()
                    .map(|f| match f {
                        FieldAssertion::Simple { field_name, .. } => field_name.clone(),
                        FieldAssertion::StructPattern { field_name, .. } => field_name.clone(),
                        FieldAssertion::TuplePattern { field_name, .. } => field_name.clone(),
                        FieldAssertion::UnitPattern { field_name, .. } => field_name.clone(),
                        #[cfg(feature = "regex")]
                        FieldAssertion::Regex { field_name, .. } => field_name.clone(),
                        FieldAssertion::Comparison { field_name, .. } => field_name.clone(),
                    })
                    .collect();

                let rest_pattern = if nested.rest {
                    quote! { , .. }
                } else {
                    quote! {}
                };

                let nested_assertions = generate_assertions(nested);

                quote! {
                    match #elem_name {
                        #struct_path { #(#field_names),* #rest_pattern } => {
                            #nested_assertions
                        },
                        _ => panic!(
                            "Element expected {}, got {:?}",
                            stringify!(#struct_path),
                            #elem_name
                        ),
                    }
                }
            } else {
                // Regular struct - use recursive assert_struct!
                let field_assignments = generate_struct_field_assignments(nested);
                let rest_pattern = if nested.rest {
                    quote! { , .. }
                } else {
                    quote! {}
                };

                quote! {
                    assert_struct!(#elem_name, #struct_path {
                        #(#field_assignments),*
                        #rest_pattern
                    });
                }
            }
        }
        PatternElement::Tuple(path, elements) => {
            // Handle nested tuples or enum variants within tuples
            if let Some(variant_path) = path {
                // It's an enum variant like Some(42) or None
                if elements.is_empty() {
                    // Unit variant like None
                    // Special handling for None
                    if is_option_none_path(variant_path) {
                        quote! {
                            match #elem_name {
                                None => {},
                                Some(_) => panic!(
                                    concat!("Element expected None, got Some")
                                ),
                            }
                        }
                    } else {
                        // General unit variant
                        quote! {
                            match #elem_name {
                                #variant_path => {},
                                _ => panic!(
                                    "Element expected {}, got {:?}",
                                    stringify!(#variant_path),
                                    #elem_name
                                ),
                            }
                        }
                    }
                } else {
                    // Tuple variant with data
                    // Special handling for Some
                    if is_option_some_path(variant_path) && elements.len() == 1 {
                        // Generate specialized Option handling
                        let inner_assertion = generate_pattern_element_assertion(
                            &quote::format_ident!("inner"),
                            &elements[0],
                        );
                        quote! {
                            match #elem_name {
                                Some(inner) => {
                                    #inner_assertion
                                },
                                None => panic!(
                                    concat!("Element expected Some(...), got None")
                                ),
                            }
                        }
                    } else {
                        // General enum tuple variant
                        let element_names: Vec<_> = (0..elements.len())
                            .map(|i| quote::format_ident!("__elem_{}", i))
                            .collect();

                        let element_assertions = elements
                            .iter()
                            .zip(&element_names)
                            .map(|(elem, name)| generate_pattern_element_assertion(name, elem))
                            .collect::<Vec<_>>();

                        quote! {
                            match #elem_name {
                                #variant_path(#(#element_names),*) => {
                                    #(#element_assertions)*
                                },
                                _ => panic!(
                                    "Element expected {}, got {:?}",
                                    stringify!(#variant_path),
                                    #elem_name
                                ),
                            }
                        }
                    }
                }
            } else {
                // Plain nested tuple
                let element_names: Vec<_> = (0..elements.len())
                    .map(|i| quote::format_ident!("__tuple_{}", i))
                    .collect();

                let destructure = quote! {
                    let (#(#element_names),*) = &#elem_name;
                };

                let element_assertions = elements
                    .iter()
                    .zip(&element_names)
                    .map(|(elem, name)| generate_pattern_element_assertion(name, elem))
                    .collect::<Vec<_>>();

                quote! {
                    {
                        #destructure
                        #(#element_assertions)*
                    }
                }
            }
        }
    }
}

/// Generate assertion for unit patterns (enum variants with no data)
fn generate_unit_pattern_assert(field_name: &syn::Ident, path: &syn::Path) -> TokenStream {
    // Special handling for None
    if is_option_none_path(path) {
        quote! {
            match #field_name {
                None => {},
                Some(_) => panic!(
                    concat!("Field `", stringify!(#field_name), "` expected None, got Some")
                ),
            }
        }
    } else {
        // General unit variant
        quote! {
            match #field_name {
                #path => {},
                _ => panic!(
                    "Field `{}` expected {}, got {:?}",
                    stringify!(#field_name),
                    stringify!(#path),
                    #field_name
                ),
            }
        }
    }
}

/// Special handling for Option::Some patterns
fn generate_option_assertion(field_name: &syn::Ident, element: &PatternElement) -> TokenStream {
    match element {
        PatternElement::Simple(expected) => {
            let transformed = transform_expected_value(expected);
            quote! {
                match #field_name {
                    Some(inner) => {
                        assert_eq!(inner, &#transformed);
                    },
                    None => panic!(
                        concat!("Field `", stringify!(#field_name), "` expected Some(...), got None")
                    ),
                }
            }
        }
        PatternElement::Comparison(op, value) => {
            let op_str = match op {
                ComparisonOp::Less => "<",
                ComparisonOp::LessEqual => "<=",
                ComparisonOp::Greater => ">",
                ComparisonOp::GreaterEqual => ">=",
            };

            let comparison = match op {
                ComparisonOp::Less => quote! { inner < &#value },
                ComparisonOp::LessEqual => quote! { inner <= &#value },
                ComparisonOp::Greater => quote! { inner > &#value },
                ComparisonOp::GreaterEqual => quote! { inner >= &#value },
            };

            quote! {
                match #field_name {
                    Some(inner) => {
                        if !(#comparison) {
                            panic!(
                                "Field `{}` failed comparison: Some({:?}) {} {}",
                                stringify!(#field_name),
                                inner,
                                #op_str,
                                &#value
                            );
                        }
                    },
                    None => panic!(
                        concat!("Field `", stringify!(#field_name), "` expected Some(...), got None")
                    ),
                }
            }
        }
        #[cfg(feature = "regex")]
        PatternElement::Regex(pattern) => {
            quote! {
                match #field_name {
                    Some(inner) => {
                        let re = ::regex::Regex::new(#pattern)
                            .expect(concat!("Invalid regex pattern: ", #pattern));
                        if !re.is_match(inner) {
                            panic!(
                                "Field `{}` does not match regex pattern `{}`\n  value: Some({:?})",
                                stringify!(#field_name),
                                #pattern,
                                inner
                            );
                        }
                    },
                    None => panic!(
                        concat!("Field `", stringify!(#field_name), "` expected Some(...), got None")
                    ),
                }
            }
        }
        PatternElement::Struct(struct_path, nested) => {
            // Some(Struct { ... }) pattern
            let field_assignments = generate_struct_field_assignments(nested);
            let rest_pattern = if nested.rest {
                quote! { , .. }
            } else {
                quote! {}
            };

            quote! {
                match #field_name {
                    Some(inner) => {
                        assert_struct!(inner, #struct_path {
                            #(#field_assignments),*
                            #rest_pattern
                        });
                    },
                    None => panic!(
                        concat!("Field `", stringify!(#field_name), "` expected Some(...), got None")
                    ),
                }
            }
        }
        PatternElement::Tuple(_path, _elements) => {
            // This shouldn't happen - Tuple patterns inside Some() should be handled differently
            // But for completeness, we can just panic with an error message
            panic!("Complex tuple patterns inside Option are not yet supported in this context");
        }
    }
}

/// Transform expected values (e.g., string literals to String)
fn transform_expected_value(expr: &Expr) -> Expr {
    match expr {
        Expr::Lit(lit) if matches!(lit.lit, syn::Lit::Str(_)) => {
            // Transform "literal" to "literal".to_string() for String fields
            syn::parse_quote! { #expr.to_string() }
        }
        _ => expr.clone(),
    }
}

/// Generate field assignments for nested struct patterns
fn generate_struct_field_assignments(nested: &Expected) -> Vec<TokenStream> {
    let mut field_assignments = Vec::new();

    for field in &nested.fields {
        match field {
            FieldAssertion::Simple {
                field_name,
                expected_value,
                ..
            } => {
                field_assignments.push(quote! {
                    #field_name: #expected_value
                });
            }
            FieldAssertion::StructPattern {
                field_name,
                path,
                nested,
                ..
            } => {
                // For nested structs in field assignments
                let nested_struct = generate_nested_struct(path, nested);
                field_assignments.push(quote! {
                    #field_name: #nested_struct
                });
            }
            FieldAssertion::TuplePattern {
                field_name,
                path,
                elements,
                ..
            } => {
                // For tuples in field assignments
                if let Some(variant_path) = path {
                    // Enum variant
                    let element_values: Vec<TokenStream> = elements
                        .iter()
                        .map(|e| match e {
                            PatternElement::Simple(expr) => quote! { #expr },
                            PatternElement::Struct(struct_path, nested) => {
                                // Generate the nested struct
                                let nested_struct = generate_nested_struct(struct_path, nested);
                                quote! { #nested_struct }
                            }
                            PatternElement::Comparison(op, value) => {
                                // For comparisons in field assignments, pass through the operator
                                let op_tokens = match op {
                                    ComparisonOp::Less => quote! { < },
                                    ComparisonOp::LessEqual => quote! { <= },
                                    ComparisonOp::Greater => quote! { > },
                                    ComparisonOp::GreaterEqual => quote! { >= },
                                };
                                quote! { #op_tokens #value }
                            }
                            #[cfg(feature = "regex")]
                            PatternElement::Regex(pattern) => {
                                quote! { =~ #pattern }
                            }
                            PatternElement::Tuple(path, inner_elements) => {
                                // For nested tuples/enums in field assignments, generate the pattern
                                if let Some(variant_path) = path {
                                    // It's an enum variant
                                    let inner_values =
                                        generate_pattern_element_values(inner_elements);
                                    quote! { #variant_path(#(#inner_values),*) }
                                } else {
                                    // Plain tuple
                                    let inner_values =
                                        generate_pattern_element_values(inner_elements);
                                    quote! { (#(#inner_values),*) }
                                }
                            }
                        })
                        .collect();
                    field_assignments.push(quote! {
                        #field_name: #variant_path(#(#element_values),*)
                    });
                } else {
                    // Plain tuple
                    let element_values: Vec<TokenStream> = elements
                        .iter()
                        .map(|e| match e {
                            PatternElement::Simple(expr) => quote! { #expr },
                            PatternElement::Struct(struct_path, nested) => {
                                // Generate the nested struct
                                let nested_struct = generate_nested_struct(struct_path, nested);
                                quote! { #nested_struct }
                            }
                            PatternElement::Comparison(op, value) => {
                                // For comparisons in field assignments, pass through the operator
                                let op_tokens = match op {
                                    ComparisonOp::Less => quote! { < },
                                    ComparisonOp::LessEqual => quote! { <= },
                                    ComparisonOp::Greater => quote! { > },
                                    ComparisonOp::GreaterEqual => quote! { >= },
                                };
                                quote! { #op_tokens #value }
                            }
                            #[cfg(feature = "regex")]
                            PatternElement::Regex(pattern) => {
                                quote! { =~ #pattern }
                            }
                            PatternElement::Tuple(path, inner_elements) => {
                                // For nested tuples/enums in field assignments, generate the pattern
                                if let Some(variant_path) = path {
                                    // It's an enum variant
                                    let inner_values =
                                        generate_pattern_element_values(inner_elements);
                                    quote! { #variant_path(#(#inner_values),*) }
                                } else {
                                    // Plain tuple
                                    let inner_values =
                                        generate_pattern_element_values(inner_elements);
                                    quote! { (#(#inner_values),*) }
                                }
                            }
                        })
                        .collect();
                    field_assignments.push(quote! {
                        #field_name: (#(#element_values),*)
                    });
                }
            }
            FieldAssertion::UnitPattern {
                field_name, path, ..
            } => {
                field_assignments.push(quote! {
                    #field_name: #path
                });
            }
            #[cfg(feature = "regex")]
            FieldAssertion::Regex {
                field_name,
                pattern,
                ..
            } => {
                // For regex in nested structs, we pass through with =~ syntax
                field_assignments.push(quote! {
                    #field_name: =~ #pattern
                });
            }
            FieldAssertion::Comparison {
                field_name,
                op,
                value,
                ..
            } => {
                // For comparisons in nested structs, we pass through the operator
                let op_tokens = match op {
                    ComparisonOp::Less => quote! { < },
                    ComparisonOp::LessEqual => quote! { <= },
                    ComparisonOp::Greater => quote! { > },
                    ComparisonOp::GreaterEqual => quote! { >= },
                };
                field_assignments.push(quote! {
                    #field_name: #op_tokens #value
                });
            }
        }
    }

    field_assignments
}

/// Generate values from pattern elements for use in field assignments
fn generate_pattern_element_values(elements: &[PatternElement]) -> Vec<TokenStream> {
    elements
        .iter()
        .map(|e| match e {
            PatternElement::Simple(expr) => quote! { #expr },
            PatternElement::Comparison(op, value) => {
                let op_tokens = match op {
                    ComparisonOp::Less => quote! { < },
                    ComparisonOp::LessEqual => quote! { <= },
                    ComparisonOp::Greater => quote! { > },
                    ComparisonOp::GreaterEqual => quote! { >= },
                };
                quote! { #op_tokens #value }
            }
            #[cfg(feature = "regex")]
            PatternElement::Regex(pattern) => {
                quote! { =~ #pattern }
            }
            PatternElement::Struct(struct_path, nested) => {
                let nested_struct = generate_nested_struct(struct_path, nested);
                quote! { #nested_struct }
            }
            PatternElement::Tuple(path, inner_elements) => {
                if let Some(variant_path) = path {
                    let inner_values = generate_pattern_element_values(inner_elements);
                    quote! { #variant_path(#(#inner_values),*) }
                } else {
                    let inner_values = generate_pattern_element_values(inner_elements);
                    quote! { (#(#inner_values),*) }
                }
            }
        })
        .collect()
}

fn generate_nested_struct(type_name: &syn::Path, expected: &Expected) -> TokenStream {
    let field_assignments = generate_struct_field_assignments(expected);

    let rest_pattern = if expected.rest {
        quote! { , .. }
    } else {
        quote! {}
    };

    quote! {
        #type_name {
            #(#field_assignments),*
            #rest_pattern
        }
    }
}

/// Check if a path refers to Option::Some
fn is_option_some_path(path: &syn::Path) -> bool {
    if let Some(segment) = path.segments.last() {
        segment.ident == "Some"
    } else {
        false
    }
}

/// Check if a path refers to Option::None
fn is_option_none_path(path: &syn::Path) -> bool {
    if let Some(segment) = path.segments.last() {
        segment.ident == "None"
    } else {
        false
    }
}
