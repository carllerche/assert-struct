use crate::{AssertStruct, ComparisonOp, FieldAssertion, Pattern};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Token, punctuated::Punctuated};

pub fn expand(assert: &AssertStruct) -> TokenStream {
    let value = &assert.value;
    let pattern = &assert.pattern;

    // Generate the assertion for the root pattern
    // Start with root path being the value identifier
    let root_path = vec![quote! { #value }.to_string()];
    let assertion =
        generate_pattern_assertion_with_path(&quote! { #value }, pattern, false, &root_path);

    // Wrap in a block to avoid variable name conflicts
    quote! {
        {
            #assertion
        }
    }
}

/// Generate assertion code for any pattern type with path tracking.
///
/// This version tracks the path to the current field for better error messages.
fn generate_pattern_assertion_with_path(
    value_expr: &TokenStream,
    pattern: &Pattern,
    is_ref: bool,
    path: &[String],
) -> TokenStream {
    // Capture pattern string representation for error messages
    let pattern_str = pattern_to_string(pattern);

    match pattern {
        Pattern::Simple(expected) => {
            // Generate simple assertion with path tracking
            generate_simple_assertion_with_path(value_expr, expected, is_ref, path)
        }
        Pattern::Struct {
            path: struct_path,
            fields,
            rest,
        } => {
            // Use the path-aware version for structs
            generate_struct_match_assertion_with_path(
                value_expr,
                struct_path,
                fields,
                *rest,
                is_ref,
                path,
            )
        }
        Pattern::Comparison(op, expected) => {
            // Generate improved comparison assertion
            generate_comparison_assertion_with_path(
                value_expr,
                op,
                expected,
                is_ref,
                path,
                &pattern_str,
            )
        }
        Pattern::Range(range) => {
            // Generate improved range assertion
            generate_range_assertion_with_path(value_expr, range, is_ref, path, &pattern_str)
        }
        Pattern::Tuple {
            path: variant_path,
            elements,
        } => {
            // Handle enum tuples with path tracking
            if let Some(vpath) = variant_path {
                if elements.is_empty() {
                    // Unit variant like None - use old version for now
                    generate_unit_variant_assertion(value_expr, vpath, is_ref)
                } else {
                    // Tuple variant with data - generate with path tracking
                    generate_enum_tuple_assertion_with_path(
                        value_expr, vpath, elements, is_ref, path,
                    )
                }
            } else {
                // Plain tuple - use old version for now
                generate_plain_tuple_assertion(value_expr, elements, is_ref)
            }
        }
        Pattern::Slice(elements) => {
            // Generate slice assertion with path tracking
            generate_slice_assertion_with_path(value_expr, elements, is_ref, path)
        }
        _ => {
            // For now, delegate other patterns to the original function
            generate_pattern_assertion(value_expr, pattern, is_ref)
        }
    }
}

/// Generate assertion code for any pattern type.
///
/// The `is_ref` parameter tracks whether `value_expr` is already a reference.
/// This is crucial for correct code generation - we need to know when to add `&`.
///
/// # Example Transformations
///
/// Simple value:
/// ```text
/// // Input: age: 30
/// // Output: assert_eq!(&value.age, &30);
/// ```
///
/// Comparison:
/// ```text
/// // Input: age: >= 18
/// // Output: assert!(value.age >= 18, "age: expected >= 18, got {:?}", value.age);
/// ```
fn generate_pattern_assertion(
    value_expr: &TokenStream,
    pattern: &Pattern,
    is_ref: bool,
) -> TokenStream {
    match pattern {
        Pattern::Simple(expected) => {
            // Direct equality check
            // Transform string literals to String for comparison with String fields
            let transformed = transform_expected_value(expected);
            if is_ref {
                // value_expr is already a reference (e.g., from destructuring)
                quote! {
                    assert_eq!(#value_expr, &#transformed);
                }
            } else {
                // value_expr needs to be referenced
                quote! {
                    assert_eq!(&#value_expr, &#transformed);
                }
            }
        }
        Pattern::Struct { path, fields, rest } => {
            // Use match expression for both structs and enums for unified handling
            // WHY: This eliminates the need for heuristics to distinguish between them.
            // The unreachable pattern warning for structs is suppressed - a small cost
            // for the robustness gain of not having to guess type categories.
            //
            // Example for struct: User { name: "Alice", age: 30 }
            // Example for enum: Status::Error { code: 500, message: "Internal" }
            // Both generate similar match expressions with exhaustive checking
            generate_struct_match_assertion(value_expr, path, fields, *rest, is_ref)
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
            // Generate comparison assertions with clear error messages
            generate_comparison_assertion(value_expr, op, value, is_ref)
        }
        Pattern::Range(range) => {
            // Use Rust's native range matching in match expressions
            // WHY: Match expressions handle all edge cases automatically
            // (reference levels, type coercion, inclusive/exclusive bounds)
            //
            // Example input: age: 18..=65
            // Generates: match &age { 18..=65 => {}, _ => panic!(...) }
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
            // PERFORMANCE OPTIMIZATION: String literal patterns compile at macro expansion
            // This path handles: email: =~ r".*@example\.com"
            // The regex compiles once at expansion time, not at runtime
            // We still use Like trait for consistency with the Like(Expr) path
            if is_ref {
                quote! {
                    {
                        use ::assert_struct::Like;
                        let re = ::regex::Regex::new(#pattern_str)
                            .expect(concat!("Invalid regex pattern: ", #pattern_str));
                        if !#value_expr.like(&re) {
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
                        use ::assert_struct::Like;
                        let re = ::regex::Regex::new(#pattern_str)
                            .expect(concat!("Invalid regex pattern: ", #pattern_str));
                        if !(&#value_expr).like(&re) {
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
        #[cfg(feature = "regex")]
        Pattern::Like(pattern_expr) => {
            // Runtime pattern matching via Like trait
            // This path handles: email: =~ my_pattern_var
            if is_ref {
                quote! {
                    {
                        use ::assert_struct::Like;
                        if !#value_expr.like(&#pattern_expr) {
                            panic!(
                                "Value does not match pattern\n  value: {:?}\n  pattern: {:?}",
                                #value_expr,
                                &#pattern_expr
                            );
                        }
                    }
                }
            } else {
                quote! {
                    {
                        use ::assert_struct::Like;
                        if !(&#value_expr).like(&#pattern_expr) {
                            panic!(
                                "Value does not match pattern\n  value: {:?}\n  pattern: {:?}",
                                &#value_expr,
                                &#pattern_expr
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

/// Generate match-based assertion for both structs and enums with fields.
///
/// Using match for both eliminates the need for type detection heuristics.
/// The `#[allow(unreachable_patterns)]` suppresses warnings for struct matches.
fn generate_struct_match_assertion_with_path(
    value_expr: &TokenStream,
    struct_path: &syn::Path,
    fields: &Punctuated<FieldAssertion, Token![,]>,
    rest: bool,
    is_ref: bool,
    field_path: &[String],
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
            // Build path for this field
            let mut new_path = field_path.to_vec();
            new_path.push(field_name.to_string());
            // Fields from destructuring are references
            generate_pattern_assertion_with_path(
                &quote! { #field_name },
                field_pattern,
                true,
                &new_path,
            )
        })
        .collect();

    if is_ref {
        quote! {
            #[allow(unreachable_patterns)]
            match #value_expr {
                #struct_path { #(#field_names),* #rest_pattern } => {
                    #(#field_assertions)*
                },
                _ => panic!(
                    "Expected {}, got {:?}",
                    stringify!(#struct_path),
                    #value_expr
                ),
            }
        }
    } else {
        quote! {
            #[allow(unreachable_patterns)]
            match &#value_expr {
                #struct_path { #(#field_names),* #rest_pattern } => {
                    #(#field_assertions)*
                },
                _ => panic!(
                    "Expected {}, got {:?}",
                    stringify!(#struct_path),
                    &#value_expr
                ),
            }
        }
    }
}

fn generate_struct_match_assertion(
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
            #[allow(unreachable_patterns)]
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
            #[allow(unreachable_patterns)]
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

/// Generate assertion for plain tuples.
///
/// # Example
/// ```text
/// // Input: point: (15, 25)
/// // Generates:
/// // let (__tuple_elem_0, __tuple_elem_1) = &point;
/// // assert_eq!(__tuple_elem_0, &15);
/// // assert_eq!(__tuple_elem_1, &25);
/// ```
fn generate_plain_tuple_assertion(
    value_expr: &TokenStream,
    elements: &[Pattern],
    is_ref: bool,
) -> TokenStream {
    // Generate unique names to avoid conflicts
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

/// Generate assertion for slice patterns using Rust's native slice matching.
///
/// # Example
/// ```text
/// // Input: values: [> 0, < 10, == 5]
/// // Generates:
/// // match values.as_slice() {
/// //     [__elem_0, __elem_1, __elem_2] => {
/// //         assert!(__elem_0 > 0);
/// //         assert!(__elem_1 < 10);
/// //         assert_eq!(__elem_2, &5);
/// //     }
/// //     _ => panic!("Pattern mismatch...")
/// // }
/// ```
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
                // Rest pattern allows variable-length matching
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

    // Convert Vec to slice for matching
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

/// Generate comparison assertion with descriptive error messages.
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

/// Transform expected values for better ergonomics.
///
/// WHY: This allows users to write `name: "Alice"` instead of `name: "Alice".to_string()`
/// when comparing against String fields. The macro automatically adds `.to_string()`
/// to string literals, making the syntax cleaner.
///
/// # Example
/// ```text
/// // User writes: name: "Alice"
/// // We transform to: name: "Alice".to_string()
/// // So it can compare with String fields
/// ```
fn transform_expected_value(expr: &Expr) -> Expr {
    match expr {
        Expr::Lit(lit) if matches!(lit.lit, syn::Lit::Str(_)) => {
            // Transform string literal to String for comparison
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

/// Convert a pattern to its string representation for error messages
fn pattern_to_string(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Simple(expr) => quote! { #expr }.to_string(),
        Pattern::Comparison(op, expr) => {
            let op_str = match op {
                ComparisonOp::Less => "<",
                ComparisonOp::LessEqual => "<=",
                ComparisonOp::Greater => ">",
                ComparisonOp::GreaterEqual => ">=",
                ComparisonOp::Equal => "==",
                ComparisonOp::NotEqual => "!=",
            };
            format!("{} {}", op_str, quote! { #expr })
        }
        Pattern::Range(range) => quote! { #range }.to_string(),
        #[cfg(feature = "regex")]
        Pattern::Regex(s) => format!("=~ r\"{}\"", s),
        #[cfg(feature = "regex")]
        Pattern::Like(expr) => format!("=~ {}", quote! { #expr }),
        Pattern::Rest => "..".to_string(),
        Pattern::Struct { path, .. } => quote! { #path { .. } }.to_string(),
        Pattern::Tuple { path, elements } => {
            if let Some(p) = path {
                if elements.is_empty() {
                    quote! { #p }.to_string()
                } else {
                    format!("{}(...)", quote! { #p })
                }
            } else {
                format!("({} elements)", elements.len())
            }
        }
        Pattern::Slice(elements) => format!("[{} elements]", elements.len()),
    }
}

/// Generate comparison assertion with enhanced error message
fn generate_comparison_assertion_with_path(
    value_expr: &TokenStream,
    op: &ComparisonOp,
    expected: &syn::Expr,
    is_ref: bool,
    path: &[String],
    pattern_str: &str,
) -> TokenStream {
    // Build the comparison expression
    let comparison = if is_ref {
        match op {
            ComparisonOp::Less => quote! { #value_expr < &(#expected) },
            ComparisonOp::LessEqual => quote! { #value_expr <= &(#expected) },
            ComparisonOp::Greater => quote! { #value_expr > &(#expected) },
            ComparisonOp::GreaterEqual => quote! { #value_expr >= &(#expected) },
            ComparisonOp::Equal => quote! { #value_expr == &(#expected) },
            ComparisonOp::NotEqual => quote! { #value_expr != &(#expected) },
        }
    } else {
        match op {
            ComparisonOp::Less => quote! { &#value_expr < &(#expected) },
            ComparisonOp::LessEqual => quote! { &#value_expr <= &(#expected) },
            ComparisonOp::Greater => quote! { &#value_expr > &(#expected) },
            ComparisonOp::GreaterEqual => quote! { &#value_expr >= &(#expected) },
            ComparisonOp::Equal => quote! { &#value_expr == &(#expected) },
            ComparisonOp::NotEqual => quote! { &#value_expr != &(#expected) },
        }
    };

    // Use improved error reporting
    let field_path = path.join(".");
    let actual_expr = if is_ref {
        quote! { #value_expr }
    } else {
        quote! { &#value_expr }
    };

    quote! {
        if !(#comparison) {
            // Capture line number using proper spanning
            let __line = line!();
            let __file = file!();

            // Build error context
            let __error = ::assert_struct::__macro_support::ErrorContext {
                field_path: #field_path.to_string(),
                pattern_str: #pattern_str.to_string(),
                actual_value: format!("{:?}", #actual_expr),
                line_number: __line,
                file_name: __file,
                error_type: ::assert_struct::__macro_support::ErrorType::Comparison,
            };

            panic!("{}", ::assert_struct::__macro_support::format_error(__error));
        }
    }
}

/// Generate assertion for enum tuple variants with path tracking
fn generate_enum_tuple_assertion_with_path(
    value_expr: &TokenStream,
    variant_path: &syn::Path,
    elements: &[Pattern],
    is_ref: bool,
    field_path: &[String],
) -> TokenStream {
    // Special handling for Some with pattern inside
    if is_option_some_path(variant_path) && elements.len() == 1 {
        // Build path for the Some content
        let mut inner_path = field_path.to_vec();
        inner_path.push("Some".to_string());

        let inner_assertion = generate_pattern_assertion_with_path(
            &quote! { inner },
            &elements[0],
            true, // inner is a reference from the match
            &inner_path,
        );

        let field_path_str = field_path.join(".");

        if is_ref {
            return quote! {
                match #value_expr {
                    Some(inner) => {
                        #inner_assertion
                    },
                    None => {
                        let __line = line!();
                        let __file = file!();
                        let __error = ::assert_struct::__macro_support::ErrorContext {
                            field_path: #field_path_str.to_string(),
                            pattern_str: "Some(...)".to_string(),
                            actual_value: "None".to_string(),
                            line_number: __line,
                            file_name: __file,
                            error_type: ::assert_struct::__macro_support::ErrorType::EnumVariant,
                        };
                        panic!("{}", ::assert_struct::__macro_support::format_error(__error));
                    }
                }
            };
        } else {
            return quote! {
                match &#value_expr {
                    Some(inner) => {
                        #inner_assertion
                    },
                    None => {
                        let __line = line!();
                        let __file = file!();
                        let __error = ::assert_struct::__macro_support::ErrorContext {
                            field_path: #field_path_str.to_string(),
                            pattern_str: "Some(...)".to_string(),
                            actual_value: "None".to_string(),
                            line_number: __line,
                            file_name: __file,
                            error_type: ::assert_struct::__macro_support::ErrorType::EnumVariant,
                        };
                        panic!("{}", ::assert_struct::__macro_support::format_error(__error));
                    }
                }
            };
        }
    }

    // For other enum tuple variants, use the old version for now
    generate_enum_tuple_assertion(value_expr, variant_path, elements, is_ref)
}

/// Generate range assertion with enhanced error message
fn generate_range_assertion_with_path(
    value_expr: &TokenStream,
    range: &syn::Expr,
    is_ref: bool,
    path: &[String],
    pattern_str: &str,
) -> TokenStream {
    let field_path = path.join(".");
    let match_expr = if is_ref {
        quote! { #value_expr }
    } else {
        quote! { &#value_expr }
    };

    quote! {
        match #match_expr {
            #range => {},
            _ => {
                // Capture line number and file info
                let __line = line!();
                let __file = file!();

                // Build error context
                let __error = ::assert_struct::__macro_support::ErrorContext {
                    field_path: #field_path.to_string(),
                    pattern_str: #pattern_str.to_string(),
                    actual_value: format!("{:?}", #match_expr),
                    line_number: __line,
                    file_name: __file,
                    error_type: ::assert_struct::__macro_support::ErrorType::Range,
                };

                panic!("{}", ::assert_struct::__macro_support::format_error(__error));
            }
        }
    }
}

/// Generate simple assertion with path tracking
fn generate_simple_assertion_with_path(
    value_expr: &TokenStream,
    expected: &syn::Expr,
    is_ref: bool,
    path: &[String],
) -> TokenStream {
    // Transform string literals for String comparison
    let transformed = transform_expected_value(expected);
    let field_path_str = path.join(".");
    let expected_str = quote! { #expected }.to_string();

    if is_ref {
        quote! {
            if #value_expr != &#transformed {
                let __line = line!();
                let __file = file!();
                let __error = ::assert_struct::__macro_support::ErrorContext {
                    field_path: #field_path_str.to_string(),
                    pattern_str: #expected_str.to_string(),
                    actual_value: format!("{:?}", #value_expr),
                    line_number: __line,
                    file_name: __file,
                    error_type: ::assert_struct::__macro_support::ErrorType::Value,
                };
                panic!("{}", ::assert_struct::__macro_support::format_error(__error));
            }
        }
    } else {
        quote! {
            if &#value_expr != &#transformed {
                let __line = line!();
                let __file = file!();
                let __error = ::assert_struct::__macro_support::ErrorContext {
                    field_path: #field_path_str.to_string(),
                    pattern_str: #expected_str.to_string(),
                    actual_value: format!("{:?}", &#value_expr),
                    line_number: __line,
                    file_name: __file,
                    error_type: ::assert_struct::__macro_support::ErrorType::Value,
                };
                panic!("{}", ::assert_struct::__macro_support::format_error(__error));
            }
        }
    }
}

/// Generate slice assertion with path tracking
fn generate_slice_assertion_with_path(
    value_expr: &TokenStream,
    elements: &[Pattern],
    _is_ref: bool,
    field_path: &[String],
) -> TokenStream {
    let mut pattern_parts = Vec::new();
    let mut bindings_and_assertions = Vec::new();

    for (i, elem) in elements.iter().enumerate() {
        match elem {
            Pattern::Rest => {
                // Rest pattern allows variable-length matching
                pattern_parts.push(quote! { .. });
            }
            _ => {
                let binding = quote::format_ident!("__elem_{}", i);
                pattern_parts.push(quote! { #binding });

                // Build path for this slice element
                let mut elem_path = field_path.to_vec();
                elem_path.push(format!("[{}]", i));

                let assertion = generate_pattern_assertion_with_path(
                    &quote! { #binding },
                    elem,
                    true, // elements from slice matching are references
                    &elem_path,
                );
                bindings_and_assertions.push(assertion);
            }
        }
    }

    // Convert Vec to slice for matching
    let slice_expr = quote! { (#value_expr).as_slice() };
    let field_path_str = field_path.join(".");
    let elements_len = elements.len();

    quote! {
        match #slice_expr {
            [#(#pattern_parts),*] => {
                #(#bindings_and_assertions)*
            }
            _ => {
                let __line = line!();
                let __file = file!();
                let __error = ::assert_struct::__macro_support::ErrorContext {
                    field_path: #field_path_str.to_string(),
                    pattern_str: format!("[{} elements]", #elements_len),
                    actual_value: format!("{:?}", &#value_expr),
                    line_number: __line,
                    file_name: __file,
                    error_type: ::assert_struct::__macro_support::ErrorType::Slice,
                };
                panic!("{}", ::assert_struct::__macro_support::format_error(__error));
            }
        }
    }
}
