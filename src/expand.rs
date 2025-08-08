use quote::quote;
use syn::Expr;
use proc_macro2::TokenStream;
use crate::{AssertStruct, Expected, FieldAssertion};

pub fn expand(assert: &AssertStruct) -> TokenStream {
    let value = &assert.value;
    let type_name = &assert.type_name;
    
    // Collect all field names for destructuring
    let field_names: Vec<_> = assert.expected.fields
        .iter()
        .map(|f| match f {
            FieldAssertion::Simple { field_name, .. } => field_name.clone(),
            FieldAssertion::Nested { field_name, .. } => field_name.clone(),
            FieldAssertion::Tuple { field_name, .. } => field_name.clone(),
        })
        .collect();
    
    // Handle partial matching with ..
    let rest_pattern = if assert.expected.rest {
        quote! { , .. }
    } else {
        quote! {}
    };
    
    // Generate the destructuring pattern with references to avoid moves
    let destructure = quote! {
        let #type_name { #(ref #field_names),* #rest_pattern } = #value;
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
            FieldAssertion::Simple { field_name, expected_value, .. } => {
                // Simple field: generate assert_eq! with reference comparison
                assertions.push(quote! {
                    assert_eq!(#field_name, &#expected_value);
                });
            }
            FieldAssertion::Nested { field_name, type_name, nested, .. } => {
                // Nested struct: recursively call assert_struct!
                let nested_assertions = generate_nested_assert(field_name, type_name, nested);
                assertions.push(nested_assertions);
            }
            FieldAssertion::Tuple { field_name, elements, .. } => {
                // Tuple: destructure and compare element by element
                let tuple_assertions = generate_tuple_assertions(field_name, elements);
                assertions.push(tuple_assertions);
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
    let destructure = quote! {
        let (#(ref #element_names),*) = #field_name;
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

fn generate_nested_assert(field_name: &syn::Ident, type_name: &syn::Path, nested: &Expected) -> TokenStream {
    // Collect field assignments for the nested struct
    let mut field_assignments = Vec::new();
    
    for field in &nested.fields {
        match field {
            FieldAssertion::Simple { field_name, expected_value, .. } => {
                field_assignments.push(quote! {
                    #field_name: #expected_value
                });
            }
            FieldAssertion::Nested { field_name, type_name, nested, .. } => {
                // For nested within nested, we need to generate the whole structure
                let nested_struct = generate_nested_struct(type_name, nested);
                field_assignments.push(quote! {
                    #field_name: #nested_struct
                });
            }
            FieldAssertion::Tuple { field_name, elements, .. } => {
                // For tuples in nested structs
                field_assignments.push(quote! {
                    #field_name: (#(#elements),*)
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
            FieldAssertion::Simple { field_name, expected_value, .. } => {
                field_assignments.push(quote! {
                    #field_name: #expected_value
                });
            }
            FieldAssertion::Nested { field_name, type_name, nested, .. } => {
                let nested_struct = generate_nested_struct(type_name, nested);
                field_assignments.push(quote! {
                    #field_name: #nested_struct
                });
            }
            FieldAssertion::Tuple { field_name, elements, .. } => {
                field_assignments.push(quote! {
                    #field_name: (#(#elements),*)
                });
            }
        }
    }
    
    quote! {
        #type_name {
            #(#field_assignments),*
        }
    }
}