use crate::{AssertStruct, ComparisonOp, Expected, FieldAssertion};
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
            FieldAssertion::Nested { field_name, .. } => field_name.clone(),
            FieldAssertion::Tuple { field_name, .. } => field_name.clone(),
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
                // field_name is already a reference from the destructuring
                assertions.push(quote! {
                    assert_eq!(#field_name, &#expected_value);
                });
            }
            FieldAssertion::Nested {
                field_name,
                type_name,
                nested,
                ..
            } => {
                // Nested struct: recursively call assert_struct!
                let nested_assertions = generate_nested_assert(field_name, type_name, nested);
                assertions.push(nested_assertions);
            }
            FieldAssertion::Tuple {
                field_name,
                elements,
                ..
            } => {
                // Tuple: destructure and compare element by element
                let tuple_assertions = generate_tuple_assertions(field_name, elements);
                assertions.push(tuple_assertions);
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

fn generate_tuple_assertions(field_name: &syn::Ident, elements: &[Expr]) -> TokenStream {
    // Generate unique names for tuple elements
    let element_names: Vec<_> = (0..elements.len())
        .map(|i| quote::format_ident!("__tuple_elem_{}", i))
        .collect();

    // Destructure the tuple
    // In Rust 2024 edition, we use & on the value instead of ref in the pattern
    let destructure = quote! {
        let (#(#element_names),*) = &#field_name;
    };

    // Generate assertions for each element
    let mut assertions = Vec::new();
    for (elem_name, expected) in element_names.iter().zip(elements.iter()) {
        assertions.push(quote! {
            assert_eq!(#elem_name, &#expected);
        });
    }

    quote! {
        {
            #destructure
            #(#assertions)*
        }
    }
}

fn generate_nested_assert(
    field_name: &syn::Ident,
    type_name: &syn::Path,
    nested: &Expected,
) -> TokenStream {
    // Collect field assignments for the nested struct
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
            FieldAssertion::Nested {
                field_name,
                type_name,
                nested,
                ..
            } => {
                // For nested within nested, we need to generate the whole structure
                let nested_struct = generate_nested_struct(type_name, nested);
                field_assignments.push(quote! {
                    #field_name: #nested_struct
                });
            }
            FieldAssertion::Tuple {
                field_name,
                elements,
                ..
            } => {
                // For tuples in nested structs
                field_assignments.push(quote! {
                    #field_name: (#(#elements),*)
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

    // Handle partial matching
    let rest_pattern = if nested.rest {
        quote! { , .. }
    } else {
        quote! {}
    };

    // Generate the recursive assert_struct! call
    quote! {
        assert_struct!(#field_name, #type_name {
            #(#field_assignments),*
            #rest_pattern
        });
    }
}

fn generate_nested_struct(type_name: &syn::Path, expected: &Expected) -> TokenStream {
    let mut field_assignments = Vec::new();

    for field in &expected.fields {
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
            FieldAssertion::Nested {
                field_name,
                type_name,
                nested,
                ..
            } => {
                let nested_struct = generate_nested_struct(type_name, nested);
                field_assignments.push(quote! {
                    #field_name: #nested_struct
                });
            }
            FieldAssertion::Tuple {
                field_name,
                elements,
                ..
            } => {
                field_assignments.push(quote! {
                    #field_name: (#(#elements),*)
                });
            }
            #[cfg(feature = "regex")]
            FieldAssertion::Regex { .. } => {
                // Regex patterns aren't used in nested struct construction
                // They're only used for assertions
                unreachable!("Regex patterns should not appear in nested struct construction")
            }
            FieldAssertion::Comparison { .. } => {
                // Comparison patterns aren't used in nested struct construction
                // They're only used for assertions
                unreachable!("Comparison patterns should not appear in nested struct construction")
            }
        }
    }

    quote! {
        #type_name {
            #(#field_assignments),*
        }
    }
}
