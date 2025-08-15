use crate::{AssertStruct, ComparisonOp, FieldAssertion, Pattern};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::{Expr, Token, punctuated::Punctuated, spanned::Spanned};

pub fn expand(assert: &AssertStruct) -> TokenStream {
    let value = &assert.value;
    let pattern = &assert.pattern;

    // Generate pattern nodes using the node IDs from the patterns
    let mut node_defs = Vec::new();
    let root_ref = generate_pattern_nodes(pattern, &mut node_defs);

    // Generate constants for all nodes
    let node_constants: Vec<TokenStream> = node_defs
        .iter()
        .map(|(id, def)| {
            let ident = Ident::new(&format!("__PATTERN_NODE_{}", id), Span::call_site());
            quote! {
                const #ident: ::assert_struct::__macro_support::PatternNode = #def;
            }
        })
        .collect();

    // Generate the assertion for the root pattern
    // Start with root path containing the variable name
    let root_path = vec![quote! { #value }.to_string()];
    let assertion =
        generate_pattern_assertion_with_collection(&quote! { #value }, pattern, false, &root_path);

    // Wrap in a block to avoid variable name conflicts
    quote! {
        {
            // Suppress clippy warnings that are expected in macro-generated code
            #[allow(clippy::neg_cmp_op_on_partial_ord, clippy::op_ref, clippy::zero_prefixed_literal)]
            let __assert_struct_result = {
                // Generate all node constants
                #(#node_constants)*

                // Store the pattern tree root
                const __PATTERN_TREE: &::assert_struct::__macro_support::PatternNode = &#root_ref;

                // Create error collection vector
                let mut __errors: Vec<::assert_struct::__macro_support::ErrorContext> = Vec::new();

                #assertion

                // Check if any errors were collected
                if !__errors.is_empty() {
                    panic!("{}", ::assert_struct::__macro_support::format_errors_with_root(__PATTERN_TREE, __errors));
                }
            };
            __assert_struct_result
        }
    }
}

/// Get the node identifier for a pattern
fn get_pattern_node_ident(pattern: &Pattern) -> Ident {
    let node_id = match pattern {
        Pattern::Simple { node_id, .. }
        | Pattern::Struct { node_id, .. }
        | Pattern::Tuple { node_id, .. }
        | Pattern::Slice { node_id, .. }
        | Pattern::Comparison { node_id, .. }
        | Pattern::Range { node_id, .. }
        | Pattern::Rest { node_id } => *node_id,
        #[cfg(feature = "regex")]
        Pattern::Regex { node_id, .. } | Pattern::Like { node_id, .. } => *node_id,
    };
    Ident::new(&format!("__PATTERN_NODE_{}", node_id), Span::call_site())
}

/// Get the span for a pattern (if available)
fn get_pattern_span(pattern: &Pattern) -> Option<Span> {
    match pattern {
        Pattern::Simple { expr, .. } => Some(expr.span()),
        Pattern::Comparison { expr, .. } => Some(expr.span()),
        Pattern::Range { expr, .. } => Some(expr.span()),
        #[cfg(feature = "regex")]
        Pattern::Regex { span, .. } => Some(*span),
        #[cfg(feature = "regex")]
        Pattern::Like { expr, .. } => Some(expr.span()),
        Pattern::Struct { path, .. } => Some(path.span()),
        Pattern::Tuple { path, .. } => path.as_ref().map(|p| p.span()),
        Pattern::Slice { .. } | Pattern::Rest { .. } => None,
    }
}

/// Generate pattern nodes using the IDs already in patterns
fn generate_pattern_nodes(
    pattern: &Pattern,
    node_defs: &mut Vec<(usize, TokenStream)>,
) -> TokenStream {
    // Get the node_id from the pattern itself
    let node_id = match pattern {
        Pattern::Simple { node_id, .. }
        | Pattern::Struct { node_id, .. }
        | Pattern::Tuple { node_id, .. }
        | Pattern::Slice { node_id, .. }
        | Pattern::Comparison { node_id, .. }
        | Pattern::Range { node_id, .. }
        | Pattern::Rest { node_id } => *node_id,
        #[cfg(feature = "regex")]
        Pattern::Regex { node_id, .. } | Pattern::Like { node_id, .. } => *node_id,
    };

    // Special handling for Rest patterns with MAX node_id (shouldn't generate constants)
    if node_id == usize::MAX {
        // For rest patterns, return inline node definition without creating a constant
        return quote! {
            ::assert_struct::__macro_support::PatternNode::Rest
        };
    }

    let node_ident = Ident::new(&format!("__PATTERN_NODE_{}", node_id), Span::call_site());

    let node_def = match pattern {
        Pattern::Simple { expr, .. } => {
            let value_str = quote! { #expr }
                .to_string()
                .replace("& [", "&[")
                .replace("& mut [", "&mut [");
            quote! {
                ::assert_struct::__macro_support::PatternNode::Simple {
                    value: #value_str,
                }
            }
        }
        Pattern::Comparison { op, expr, .. } => {
            let op_str = match op {
                ComparisonOp::Less => "<",
                ComparisonOp::LessEqual => "<=",
                ComparisonOp::Greater => ">",
                ComparisonOp::GreaterEqual => ">=",
                ComparisonOp::Equal => "==",
                ComparisonOp::NotEqual => "!=",
            };
            let value_str = quote! { #expr }.to_string();
            quote! {
                ::assert_struct::__macro_support::PatternNode::Comparison {
                    op: #op_str,
                    value: #value_str,
                }
            }
        }
        Pattern::Range { expr, .. } => {
            let pattern_str = quote! { #expr }.to_string();
            quote! {
                ::assert_struct::__macro_support::PatternNode::Range {
                    pattern: #pattern_str,
                }
            }
        }
        #[cfg(feature = "regex")]
        Pattern::Regex { pattern, .. } => {
            let pattern_str = format!("r\"{}\"", pattern);
            quote! {
                ::assert_struct::__macro_support::PatternNode::Regex {
                    pattern: #pattern_str,
                }
            }
        }
        #[cfg(feature = "regex")]
        Pattern::Like { expr, .. } => {
            let expr_str = quote! { #expr }.to_string();
            quote! {
                ::assert_struct::__macro_support::PatternNode::Like {
                    expr: #expr_str,
                }
            }
        }
        Pattern::Rest { .. } => {
            quote! {
                ::assert_struct::__macro_support::PatternNode::Rest
            }
        }
        Pattern::Tuple { path, elements, .. } => {
            let child_refs: Vec<TokenStream> = elements
                .iter()
                .map(|elem| generate_pattern_nodes(elem, node_defs))
                .collect();

            if let Some(enum_path) = path {
                let path_str = quote!(#enum_path).to_string().replace(" :: ", "::");
                quote! {
                    ::assert_struct::__macro_support::PatternNode::EnumVariant {
                        path: #path_str,
                        args: Some(&[#(&#child_refs),*]),
                    }
                }
            } else {
                quote! {
                    ::assert_struct::__macro_support::PatternNode::Tuple {
                        items: &[#(&#child_refs),*],
                    }
                }
            }
        }
        Pattern::Slice { elements, .. } => {
            let child_refs: Vec<TokenStream> = elements
                .iter()
                .map(|elem| generate_pattern_nodes(elem, node_defs))
                .collect();

            let is_ref = true; // Default for now
            quote! {
                ::assert_struct::__macro_support::PatternNode::Slice {
                    items: &[#(&#child_refs),*],
                    is_ref: #is_ref,
                }
            }
        }
        Pattern::Struct {
            path, fields, rest, ..
        } => {
            // Fix: Remove spaces around :: when converting path to string
            let name_str = quote! { #path }.to_string().replace(" :: ", "::");

            let field_entries: Vec<TokenStream> = fields
                .iter()
                .map(|field| {
                    let field_name = field.field_name.to_string();
                    let child_ref = generate_pattern_nodes(&field.pattern, node_defs);
                    quote! {
                        (#field_name, &#child_ref)
                    }
                })
                .collect();

            if *rest {
                // Rest patterns are handled inline, no need for a separate node
                quote! {
                    ::assert_struct::__macro_support::PatternNode::Struct {
                        name: #name_str,
                        fields: &[
                            #(#field_entries,)*
                            ("..", &::assert_struct::__macro_support::PatternNode::Rest)
                        ],
                    }
                }
            } else {
                quote! {
                    ::assert_struct::__macro_support::PatternNode::Struct {
                        name: #name_str,
                        fields: &[#(#field_entries),*],
                    }
                }
            }
        }
    };

    node_defs.push((node_id, node_def));
    quote! { #node_ident }
}

/// Generate assertion code with error collection instead of immediate panic.
fn generate_pattern_assertion_with_collection(
    value_expr: &TokenStream,
    pattern: &Pattern,
    is_ref: bool,
    path: &[String],
) -> TokenStream {
    // Capture pattern string representation for error messages
    let pattern_str = pattern_to_string(pattern);

    // Get the node identifier for this pattern
    let node_ident = get_pattern_node_ident(pattern);

    match pattern {
        Pattern::Simple { expr: expected, .. } => generate_simple_assertion_with_collection(
            value_expr,
            expected,
            is_ref,
            path,
            &node_ident,
        ),
        Pattern::Struct {
            path: struct_path,
            fields,
            rest,
            ..
        } => generate_struct_match_assertion_with_collection(
            value_expr,
            struct_path,
            fields,
            *rest,
            is_ref,
            path,
            &node_ident,
        ),
        Pattern::Comparison {
            op, expr: expected, ..
        } => generate_comparison_assertion_with_collection(
            value_expr,
            op,
            expected,
            is_ref,
            path,
            &pattern_str,
            &node_ident,
        ),
        Pattern::Tuple {
            path: variant_path,
            elements,
            ..
        } => {
            // Handle enum tuples with error collection
            if let Some(vpath) = variant_path {
                if elements.is_empty() {
                    // Unit variant like None - use unified tuple function with empty elements
                    generate_enum_tuple_assertion_with_path(
                        value_expr,
                        vpath,
                        &[], // Empty elements for unit variant
                        is_ref,
                        path,
                        &node_ident,
                    )
                } else {
                    // Tuple variant with data - use collection version
                    generate_enum_tuple_assertion_with_collection(
                        value_expr,
                        vpath,
                        elements,
                        is_ref,
                        path,
                        &node_ident,
                    )
                }
            } else {
                // Plain tuple - for now use path version
                generate_plain_tuple_assertion_with_path(
                    value_expr,
                    elements,
                    is_ref,
                    path,
                    &node_ident,
                )
            }
        }
        // For now, use immediate panic for other patterns - can implement collection later
        _ => {
            // Note: this doesn't collect errors but ensures compilation
            generate_pattern_assertion_with_path(value_expr, pattern, is_ref, path)
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

    // Get the node identifier for this pattern
    let node_ident = get_pattern_node_ident(pattern);

    match pattern {
        Pattern::Simple { expr: expected, .. } => {
            // Generate simple assertion with path tracking
            generate_simple_assertion_with_path(value_expr, expected, is_ref, path, &node_ident)
        }
        Pattern::Struct {
            path: struct_path,
            fields,
            rest,
            ..
        } => {
            // Use the path-aware version for structs
            generate_struct_match_assertion_with_path(
                value_expr,
                struct_path,
                fields,
                *rest,
                is_ref,
                path,
                &node_ident,
            )
        }
        Pattern::Comparison {
            op, expr: expected, ..
        } => {
            // Generate improved comparison assertion
            generate_comparison_assertion_with_node(
                value_expr,
                op,
                expected,
                is_ref,
                path,
                &pattern_str,
                &node_ident,
            )
        }
        Pattern::Range { expr: range, .. } => {
            // Generate improved range assertion
            generate_range_assertion_with_path(
                value_expr,
                range,
                is_ref,
                path,
                &pattern_str,
                &node_ident,
            )
        }
        Pattern::Tuple {
            path: variant_path,
            elements,
            ..
        } => {
            // Handle enum tuples with path tracking
            if let Some(vpath) = variant_path {
                if elements.is_empty() {
                    // Unit variant like None - use unified tuple function with empty elements
                    generate_enum_tuple_assertion_with_path(
                        value_expr,
                        vpath,
                        &[], // Empty elements for unit variant
                        is_ref,
                        path,
                        &node_ident,
                    )
                } else {
                    // Tuple variant with data - generate with path tracking
                    generate_enum_tuple_assertion_with_path(
                        value_expr,
                        vpath,
                        elements,
                        is_ref,
                        path,
                        &node_ident,
                    )
                }
            } else {
                // Plain tuple - use path-aware version
                generate_plain_tuple_assertion_with_path(
                    value_expr,
                    elements,
                    is_ref,
                    path,
                    &node_ident,
                )
            }
        }
        Pattern::Slice { elements, .. } => {
            // Generate slice assertion with path tracking
            generate_slice_assertion_with_path(value_expr, elements, is_ref, path, &node_ident)
        }
        #[cfg(feature = "regex")]
        Pattern::Regex {
            pattern: regex_str,
            span,
            ..
        } => {
            // Generate regex assertion with path tracking
            generate_regex_assertion_with_path(
                value_expr,
                regex_str,
                *span,
                is_ref,
                path,
                &pattern_str,
                &node_ident,
            )
        }
        #[cfg(feature = "regex")]
        Pattern::Like {
            expr: pattern_expr, ..
        } => {
            // Generate Like trait assertion with path tracking
            generate_like_assertion_with_path(value_expr, pattern_expr, is_ref, path, &node_ident)
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
        Pattern::Simple { expr: expected, .. } => {
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
        Pattern::Struct {
            path, fields, rest, ..
        } => {
            // Use match expression for both structs and enums for unified handling
            // WHY: This eliminates the need for heuristics to distinguish between them.
            // The unreachable pattern warning for structs is suppressed - a small cost
            // for the robustness gain of not having to guess type categories.
            //
            // Example for struct: User { name: "Alice", age: 30 }
            // Example for enum: Status::Error { code: 500, message: "Internal" }
            // Both generate similar match expressions with exhaustive checking
            // Use dummy values for basic pattern assertion (no error collection)
            let dummy_node = quote::format_ident!("__DUMMY_NODE");
            generate_struct_match_assertion_with_path(
                value_expr,
                path,
                fields,
                *rest,
                is_ref,
                &[],
                &dummy_node,
            )
        }
        Pattern::Tuple { path, elements, .. } => {
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
        Pattern::Slice { elements, .. } => {
            // Slice pattern using Rust's native slice matching
            generate_slice_assertion(value_expr, elements, is_ref)
        }
        Pattern::Comparison {
            op, expr: value, ..
        } => {
            // Generate comparison assertions with clear error messages
            generate_comparison_assertion(value_expr, op, value, is_ref)
        }
        Pattern::Range { expr: range, .. } => {
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
        Pattern::Regex {
            pattern: pattern_str,
            span: pattern_span,
            ..
        } => {
            // PERFORMANCE OPTIMIZATION: String literal patterns compile at macro expansion
            // This path handles: email: =~ r".*@example\.com"
            // The regex compiles once at expansion time, not at runtime
            // We still use Like trait for consistency with the Like(Expr) path
            if is_ref {
                quote_spanned! {*pattern_span=>
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
                quote_spanned! {*pattern_span=>
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
        Pattern::Like {
            expr: pattern_expr, ..
        } => {
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
        Pattern::Rest { .. } => {
            // Rest patterns don't generate assertions themselves
            quote! {}
        }
    }
}

// Generate assertion for unit variants (old version without path tracking)
fn generate_unit_variant_assertion(
    value_expr: &TokenStream,
    path: &syn::Path,
    is_ref: bool,
) -> TokenStream {
    // Generic handling for all enum unit variants
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

/// Generate struct assertion with error collection for multiple field failures
fn generate_struct_match_assertion_with_collection(
    value_expr: &TokenStream,
    struct_path: &syn::Path,
    fields: &Punctuated<FieldAssertion, Token![,]>,
    rest: bool,
    is_ref: bool,
    field_path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    let field_names: Vec<_> = fields.iter().map(|f| &f.field_name).collect();
    let field_path_str = field_path.join(".");

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
            let assertion = generate_pattern_assertion_with_collection(
                &quote! { #field_name },
                field_pattern,
                true,
                &new_path,
            );

            // Wrap the assertion with the span of the field pattern if available
            if let Some(span) = get_pattern_span(field_pattern) {
                quote_spanned! {span=> #assertion }
            } else {
                assertion
            }
        })
        .collect();

    if is_ref {
        quote! {
            #[allow(unreachable_patterns)]
            match #value_expr {
                #struct_path { #(#field_names),* #rest_pattern } => {
                    #(#field_assertions)*
                },
                _ => {
                    let __line = line!();
                    let __file = file!();



                    let __error = ::assert_struct::__macro_support::ErrorContext {
                        field_path: #field_path_str.to_string(),
                        pattern_str: stringify!(#struct_path).to_string(),
                        actual_value: format!("{:?}", #value_expr),
                        line_number: __line,
                        file_name: __file,
                        error_type: ::assert_struct::__macro_support::ErrorType::EnumVariant,

                        expected_value: None,

                        error_node: Some(&#node_ident),
                    };
                    __errors.push(__error);
                }
            }
        }
    } else {
        quote! {
            #[allow(unreachable_patterns)]
            match &#value_expr {
                #struct_path { #(#field_names),* #rest_pattern } => {
                    #(#field_assertions)*
                },
                _ => {
                    let __line = line!();
                    let __file = file!();



                    let __error = ::assert_struct::__macro_support::ErrorContext {
                        field_path: #field_path_str.to_string(),
                        pattern_str: stringify!(#struct_path).to_string(),
                        actual_value: format!("{:?}", &#value_expr),
                        line_number: __line,
                        file_name: __file,
                        error_type: ::assert_struct::__macro_support::ErrorType::EnumVariant,

                        expected_value: None,

                        error_node: Some(&#node_ident),
                    };
                    __errors.push(__error);
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
    node_ident: &Ident,
) -> TokenStream {
    let field_names: Vec<_> = fields.iter().map(|f| &f.field_name).collect();
    let field_path_str = field_path.join(".");

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
                _ => {
                    let __line = line!();
                    let __file = file!();



                    let __error = ::assert_struct::__macro_support::ErrorContext {
                        field_path: #field_path_str.to_string(),
                        pattern_str: stringify!(#struct_path).to_string(),
                        actual_value: format!("{:?}", #value_expr),
                        line_number: __line,
                        file_name: __file,
                        error_type: ::assert_struct::__macro_support::ErrorType::EnumVariant,

                        expected_value: None,

                        error_node: Some(&#node_ident),
                    };
                    panic!("{}", ::assert_struct::__macro_support::format_errors_with_root(__PATTERN_TREE, vec![__error]));
                }
            }
        }
    } else {
        quote! {
            #[allow(unreachable_patterns)]
            match &#value_expr {
                #struct_path { #(#field_names),* #rest_pattern } => {
                    #(#field_assertions)*
                },
                _ => {
                    let __line = line!();
                    let __file = file!();



                    let __error = ::assert_struct::__macro_support::ErrorContext {
                        field_path: #field_path_str.to_string(),
                        pattern_str: stringify!(#struct_path).to_string(),
                        actual_value: format!("{:?}", &#value_expr),
                        line_number: __line,
                        file_name: __file,
                        error_type: ::assert_struct::__macro_support::ErrorType::EnumVariant,

                        expected_value: None,

                        error_node: Some(&#node_ident),
                    };
                    panic!("{}", ::assert_struct::__macro_support::format_errors_with_root(__PATTERN_TREE, vec![__error]));
                }
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
    // General enum tuple variant (handles all enums including Option::Some)
    let element_names: Vec<_> = (0..elements.len())
        .map(|i| quote::format_ident!("__elem_{}", i))
        .collect();

    let element_assertions: Vec<_> = element_names
        .iter()
        .zip(elements)
        .map(|(name, pattern)| {
            // Use the with_path version which supports tree-based formatting
            generate_pattern_assertion_with_path(&quote! { #name }, pattern, true, &[])
        })
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

/// Generate assertion for plain tuples with path tracking.
fn generate_plain_tuple_assertion_with_path(
    value_expr: &TokenStream,
    elements: &[Pattern],
    is_ref: bool,
    field_path: &[String],
    _node_ident: &Ident,
) -> TokenStream {
    // Generate unique names to avoid conflicts
    let element_names: Vec<_> = (0..elements.len())
        .map(|i| quote::format_ident!("__tuple_elem_{}", i))
        .collect();

    let element_assertions: Vec<_> = element_names
        .iter()
        .zip(elements)
        .enumerate()
        .map(|(i, (name, pattern))| {
            // Build path for this tuple element
            let mut elem_path = field_path.to_vec();
            // Add the index as a separate path component
            elem_path.push(i.to_string());
            generate_pattern_assertion_with_path(&quote! { #name }, pattern, true, &elem_path)
        })
        .collect();

    // Generate the destructuring and assertions
    if is_ref {
        quote! {
            {
                let (#(#element_names),*) = #value_expr;
                #(#element_assertions)*
            }
        }
    } else {
        quote! {
            {
                let (#(#element_names),*) = &#value_expr;
                #(#element_assertions)*
            }
        }
    }
}

/// Generate assertion for plain tuples (old version without path tracking).
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
        .map(|(name, pattern)| {
            // Use the with_path version which supports tree-based formatting
            generate_pattern_assertion_with_path(&quote! { #name }, pattern, true, &[])
        })
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
            Pattern::Rest { .. } => {
                // Rest pattern allows variable-length matching
                pattern_parts.push(quote! { .. });
            }
            _ => {
                let binding = quote::format_ident!("__elem_{}", i);
                pattern_parts.push(quote! { #binding });

                // Use the with_path version which supports tree-based formatting
                let assertion =
                    generate_pattern_assertion_with_path(&quote! { #binding }, elem, true, &[]);
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

/// Convert a pattern to its string representation for error messages
fn pattern_to_string(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Simple { expr, .. } => quote! { #expr }.to_string(),
        Pattern::Comparison { op, expr, .. } => {
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
        Pattern::Range { expr: range, .. } => quote! { #range }.to_string(),
        #[cfg(feature = "regex")]
        Pattern::Regex { pattern, .. } => format!("=~ r\"{}\"", pattern),
        #[cfg(feature = "regex")]
        Pattern::Like { expr, .. } => format!("=~ {}", quote! { #expr }),
        Pattern::Rest { .. } => "..".to_string(),
        Pattern::Struct { path, .. } => quote! { #path { .. } }.to_string(),
        Pattern::Tuple { path, elements, .. } => {
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
        Pattern::Slice { elements, .. } => format!("[{} elements]", elements.len()),
    }
}

/// Generate comparison assertion with error collection
fn generate_comparison_assertion_with_collection(
    value_expr: &TokenStream,
    op: &ComparisonOp,
    expected: &Expr,
    is_ref: bool,
    path: &[String],
    pattern_str: &str,
    node_ident: &Ident,
) -> TokenStream {
    let field_path = path.join(".");

    // Adjust for reference level
    let actual_expr = if is_ref {
        quote! { #value_expr }
    } else {
        quote! { &#value_expr }
    };

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

    let error_type = if matches!(op, ComparisonOp::Equal) {
        quote! { ::assert_struct::__macro_support::ErrorType::Equality }
    } else {
        quote! { ::assert_struct::__macro_support::ErrorType::Comparison }
    };

    let expected_value = if matches!(op, ComparisonOp::Equal) {
        quote! { Some(format!("{:?}", #expected)) }
    } else {
        quote! { None }
    };

    let span = expected.span();
    quote_spanned! {span=>
        if !(#comparison) {
            // Capture line number using proper spanning
            let __line = line!();
            let __file = file!();

            // Build error context
            let mut __error = ::assert_struct::__macro_support::ErrorContext {
                field_path: #field_path.to_string(),
                pattern_str: #pattern_str.to_string(),
                actual_value: format!("{:?}", #actual_expr),
                line_number: __line,
                file_name: __file,
                error_type: #error_type,

                expected_value: None,

                error_node: Some(&#node_ident),
            };

            // Add expected value for equality patterns
            if let Some(expected) = #expected_value {
                __error.expected_value = Some(expected);
            }

            __errors.push(__error);
        }
    }
}

/// Generate comparison assertion with node reference for tree-based error messages
fn generate_comparison_assertion_with_node(
    value_expr: &TokenStream,
    op: &ComparisonOp,
    expected: &Expr,
    is_ref: bool,
    path: &[String],
    pattern_str: &str,
    node_ident: &Ident,
) -> TokenStream {
    let field_path = path.join(".");

    // Adjust for reference level
    let actual_expr = if is_ref {
        quote! { #value_expr }
    } else {
        quote! { &#value_expr }
    };

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

    let error_type = if matches!(op, ComparisonOp::Equal) {
        quote! { ::assert_struct::__macro_support::ErrorType::Equality }
    } else {
        quote! { ::assert_struct::__macro_support::ErrorType::Comparison }
    };

    let expected_value = if matches!(op, ComparisonOp::Equal) {
        quote! { Some(format!("{:?}", #expected)) }
    } else {
        quote! { None }
    };

    let span = expected.span();
    quote_spanned! {span=>
        if !(#comparison) {
            // Capture line number using proper spanning
            let __line = line!();
            let __file = file!();

            // Build error context
            let mut __error = ::assert_struct::__macro_support::ErrorContext {
                field_path: #field_path.to_string(),
                pattern_str: #pattern_str.to_string(),
                actual_value: format!("{:?}", #actual_expr),
                line_number: __line,
                file_name: __file,
                error_type: #error_type,

                expected_value: None,

                error_node: Some(&#node_ident),
            };

            // Add expected value for equality patterns
            if let Some(expected) = #expected_value {
                __error.expected_value = Some(expected);
            }

            panic!("{}", ::assert_struct::__macro_support::format_errors_with_root(__PATTERN_TREE, vec![__error]));
        }
    }
}

/// Generate assertion for enum tuple variants with error collection
fn generate_enum_tuple_assertion_with_collection(
    value_expr: &TokenStream,
    variant_path: &syn::Path,
    elements: &[Pattern],
    is_ref: bool,
    field_path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    // Generic handling for all enum tuple variants with error collection
    let element_names: Vec<_> = (0..elements.len())
        .map(|i| quote::format_ident!("__elem_{}", i))
        .collect();

    // Extract variant name from path for better error messages
    let variant_name = if let Some(segment) = variant_path.segments.last() {
        segment.ident.to_string()
    } else {
        "variant".to_string()
    };

    let element_assertions: Vec<_> = element_names
        .iter()
        .zip(elements)
        .enumerate()
        .map(|(i, (name, pattern))| {
            // Build path for this tuple element
            let mut elem_path = field_path.to_vec();
            // For single-element tuple variants, use the variant name for better error messages
            // For multi-element variants, use indices
            if elements.len() == 1 {
                elem_path.push(variant_name.clone());
            } else {
                elem_path.push(i.to_string());
            }
            // Use with_collection for error collection
            generate_pattern_assertion_with_collection(&quote! { #name }, pattern, true, &elem_path)
        })
        .collect();

    let field_path_str = field_path.join(".");
    let span = variant_path.span();

    if is_ref {
        quote_spanned! {span=>
            match #value_expr {
                #variant_path(#(#element_names),*) => {
                    #(#element_assertions)*
                },
                _ => {
                    let __line = line!();
                    let __file = file!();

                    // Format the actual value more concisely for enum variants
                    let __actual_str = {
                        let debug_str = format!("{:?}", #value_expr);
                        // Try to extract just the variant name with (..) for better readability
                        if let Some(paren_pos) = debug_str.find('(') {
                            if let Some(variant_end) = debug_str[..paren_pos].rfind(|c: char| c.is_alphabetic() || c == '_') {
                                // Find the start of the variant name (after :: or at beginning)
                                let variant_start = debug_str[..=variant_end].rfind("::").map(|i| i + 2).unwrap_or(0);
                                format!("{}(..)", &debug_str[variant_start..paren_pos])
                            } else {
                                debug_str
                            }
                        } else if let Some(brace_pos) = debug_str.find('{') {
                            // Struct variant
                            if let Some(variant_end) = debug_str[..brace_pos].rfind(|c: char| c.is_alphabetic() || c == '_') {
                                let variant_start = debug_str[..=variant_end].rfind("::").map(|i| i + 2).unwrap_or(0);
                                format!("{} {{ .. }}", &debug_str[variant_start..brace_pos].trim())
                            } else {
                                debug_str
                            }
                        } else {
                            // Unit variant or simple value - use as is
                            debug_str
                        }
                    };

                    let __error = ::assert_struct::__macro_support::ErrorContext {
                        field_path: #field_path_str.to_string(),
                        pattern_str: stringify!(#variant_path).to_string(),
                        actual_value: __actual_str,
                        line_number: __line,
                        file_name: __file,
                        error_type: ::assert_struct::__macro_support::ErrorType::EnumVariant,


                        expected_value: None,

                        error_node: Some(&#node_ident),
                    };
                    __errors.push(__error);
                }
            }
        }
    } else {
        quote_spanned! {span=>
            match &#value_expr {
                #variant_path(#(#element_names),*) => {
                    #(#element_assertions)*
                },
                _ => {
                    let __line = line!();
                    let __file = file!();
                    let __error = ::assert_struct::__macro_support::ErrorContext {
                        field_path: #field_path_str.to_string(),
                        pattern_str: stringify!(#variant_path).to_string(),
                        actual_value: format!("{:?}", &#value_expr),
                        line_number: __line,
                        file_name: __file,
                        error_type: ::assert_struct::__macro_support::ErrorType::EnumVariant,


                        expected_value: None,

                        error_node: Some(&#node_ident),
                    };
                    __errors.push(__error);
                }
            }
        }
    }
}

/// Generate assertion for enum variants with path tracking (both unit and tuple)
fn generate_enum_tuple_assertion_with_path(
    value_expr: &TokenStream,
    variant_path: &syn::Path,
    elements: &[Pattern],
    is_ref: bool,
    field_path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    // Unified handling for all enum variants
    // Empty elements = unit variant (e.g., None, Status::Active)
    // Non-empty = tuple variant (e.g., Some(42), Event::Click(x, y))

    let field_path_str = field_path.join(".");
    let span = variant_path.span();

    // Generate match pattern and assertions based on whether we have elements
    let (match_pattern, inner_assertions) = if elements.is_empty() {
        // Unit variant - no bindings or inner assertions
        (quote! { #variant_path }, quote! {})
    } else {
        // Tuple variant with elements
        let element_names: Vec<_> = (0..elements.len())
            .map(|i| quote::format_ident!("__elem_{}", i))
            .collect();

        // Extract variant name from path for better error messages
        let variant_name = if let Some(segment) = variant_path.segments.last() {
            segment.ident.to_string()
        } else {
            "variant".to_string()
        };

        let element_assertions: Vec<_> = element_names
            .iter()
            .zip(elements)
            .enumerate()
            .map(|(i, (name, pattern))| {
                // Build path for this tuple element
                let mut elem_path = field_path.to_vec();
                // For single-element tuple variants, use the variant name for better error messages
                // For multi-element variants, use indices
                if elements.len() == 1 {
                    elem_path.push(variant_name.clone());
                } else {
                    elem_path.push(i.to_string());
                }

                generate_pattern_assertion_with_path(&quote! { #name }, pattern, true, &elem_path)
            })
            .collect();

        (
            quote! { #variant_path(#(#element_names),*) },
            quote! { #(#element_assertions)* },
        )
    };

    if is_ref {
        quote_spanned! {span=>
            match #value_expr {
                #match_pattern => {
                    #inner_assertions
                },
                _ => {
                    let __line = line!();
                    let __file = file!();

                    // Format the actual value more concisely for enum variants
                    let __actual_str = {
                        let debug_str = format!("{:?}", #value_expr);
                        // Try to extract just the variant name with (..) for better readability
                        if let Some(paren_pos) = debug_str.find('(') {
                            if let Some(variant_end) = debug_str[..paren_pos].rfind(|c: char| c.is_alphabetic() || c == '_') {
                                // Find the start of the variant name (after :: or at beginning)
                                let variant_start = debug_str[..=variant_end].rfind("::").map(|i| i + 2).unwrap_or(0);
                                format!("{}(..)", &debug_str[variant_start..paren_pos])
                            } else {
                                debug_str
                            }
                        } else if let Some(brace_pos) = debug_str.find('{') {
                            // Struct variant
                            if let Some(variant_end) = debug_str[..brace_pos].rfind(|c: char| c.is_alphabetic() || c == '_') {
                                let variant_start = debug_str[..=variant_end].rfind("::").map(|i| i + 2).unwrap_or(0);
                                format!("{} {{ .. }}", &debug_str[variant_start..brace_pos].trim())
                            } else {
                                debug_str
                            }
                        } else {
                            // Unit variant or simple value - use as is
                            debug_str
                        }
                    };

                    let __error = ::assert_struct::__macro_support::ErrorContext {
                        field_path: #field_path_str.to_string(),
                        pattern_str: stringify!(#variant_path).to_string(),
                        actual_value: __actual_str,
                        line_number: __line,
                        file_name: __file,
                        error_type: ::assert_struct::__macro_support::ErrorType::EnumVariant,


                        expected_value: None,

                        error_node: Some(&#node_ident),
                    };
                    panic!("{}", ::assert_struct::__macro_support::format_errors_with_root(__PATTERN_TREE, vec![__error]));
                }
            }
        }
    } else {
        quote_spanned! {span=>
            match &#value_expr {
                #match_pattern => {
                    #inner_assertions
                },
                _ => {
                    let __line = line!();
                    let __file = file!();

                    // Format the actual value more concisely for enum variants
                    let __actual_str = {
                        let debug_str = format!("{:?}", &#value_expr);
                        // Try to extract just the variant name with (..) for better readability
                        if let Some(paren_pos) = debug_str.find('(') {
                            if let Some(variant_end) = debug_str[..paren_pos].rfind(|c: char| c.is_alphabetic() || c == '_') {
                                // Find the start of the variant name (after :: or at beginning)
                                let variant_start = debug_str[..=variant_end].rfind("::").map(|i| i + 2).unwrap_or(0);
                                format!("{}(..)", &debug_str[variant_start..paren_pos])
                            } else {
                                debug_str
                            }
                        } else if let Some(brace_pos) = debug_str.find('{') {
                            // Struct variant
                            if let Some(variant_end) = debug_str[..brace_pos].rfind(|c: char| c.is_alphabetic() || c == '_') {
                                let variant_start = debug_str[..=variant_end].rfind("::").map(|i| i + 2).unwrap_or(0);
                                format!("{} {{ .. }}", &debug_str[variant_start..brace_pos].trim())
                            } else {
                                debug_str
                            }
                        } else {
                            // Unit variant or simple value - use as is
                            debug_str
                        }
                    };

                    let __error = ::assert_struct::__macro_support::ErrorContext {
                        field_path: #field_path_str.to_string(),
                        pattern_str: stringify!(#variant_path).to_string(),
                        actual_value: __actual_str,
                        line_number: __line,
                        file_name: __file,
                        error_type: ::assert_struct::__macro_support::ErrorType::EnumVariant,


                        expected_value: None,

                        error_node: Some(&#node_ident),
                    };
                    panic!("{}", ::assert_struct::__macro_support::format_errors_with_root(__PATTERN_TREE, vec![__error]));
                }
            }
        }
    }
}

/// Generate range assertion with enhanced error message
fn generate_range_assertion_with_path(
    value_expr: &TokenStream,
    range: &syn::Expr,
    is_ref: bool,
    path: &[String],
    pattern_str: &str,
    node_ident: &Ident,
) -> TokenStream {
    let field_path = path.join(".");
    let match_expr = if is_ref {
        quote! { #value_expr }
    } else {
        quote! { &#value_expr }
    };

    let span = range.span();
    quote_spanned! {span=>
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

                expected_value: None,

                error_node: Some(&#node_ident),
                };

                panic!("{}", ::assert_struct::__macro_support::format_errors_with_root(__PATTERN_TREE, vec![__error]));
            }
        }
    }
}

/// Generate simple assertion with error collection
fn generate_simple_assertion_with_collection(
    value_expr: &TokenStream,
    expected: &syn::Expr,
    is_ref: bool,
    path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    // Transform string literals for String comparison
    let transformed = transform_expected_value(expected);
    let field_path_str = path.join(".");
    let expected_str = quote! { #expected }.to_string();

    let span = expected.span();
    if is_ref {
        quote_spanned! {span=>
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

                expected_value: None,

                error_node: Some(&#node_ident),
                };
                __errors.push(__error);
            }
        }
    } else {
        quote_spanned! {span=>
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

                expected_value: None,

                error_node: Some(&#node_ident),
                };
                __errors.push(__error);
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
    node_ident: &Ident,
) -> TokenStream {
    // Transform string literals for String comparison
    let transformed = transform_expected_value(expected);
    let field_path_str = path.join(".");
    let expected_str = quote! { #expected }.to_string();

    let span = expected.span();
    if is_ref {
        quote_spanned! {span=>
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

                expected_value: None,

                error_node: Some(&#node_ident),
                };
                panic!("{}", ::assert_struct::__macro_support::format_errors_with_root(__PATTERN_TREE, vec![__error]));
            }
        }
    } else {
        quote_spanned! {span=>
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

                expected_value: None,

                error_node: Some(&#node_ident),
                };
                panic!("{}", ::assert_struct::__macro_support::format_errors_with_root(__PATTERN_TREE, vec![__error]));
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
    node_ident: &Ident,
) -> TokenStream {
    let mut pattern_parts = Vec::new();
    let mut bindings_and_assertions = Vec::new();

    for (i, elem) in elements.iter().enumerate() {
        match elem {
            Pattern::Rest { .. } => {
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
                expected_value: None,

                error_node: Some(&#node_ident),
                };
                panic!("{}", ::assert_struct::__macro_support::format_errors_with_root(__PATTERN_TREE, vec![__error]));
            }
        }
    }
}

#[cfg(feature = "regex")]
/// Generate regex assertion with path tracking
fn generate_regex_assertion_with_path(
    value_expr: &TokenStream,
    pattern_str: &str,
    span: proc_macro2::Span,
    is_ref: bool,
    path: &[String],
    full_pattern_str: &str,
    node_ident: &Ident,
) -> TokenStream {
    let field_path_str = path.join(".");

    if is_ref {
        quote_spanned! {span=>
            {
                use ::assert_struct::Like;
                let re = ::regex::Regex::new(#pattern_str)
                    .expect(concat!("Invalid regex pattern: ", #pattern_str));
                if !#value_expr.like(&re) {
                    let __line = line!();
                    let __file = file!();
                    let __error = ::assert_struct::__macro_support::ErrorContext {
                        field_path: #field_path_str.to_string(),
                        pattern_str: #full_pattern_str.to_string(),
                        actual_value: format!("{:?}", #value_expr),
                        line_number: __line,
                        file_name: __file,
                        error_type: ::assert_struct::__macro_support::ErrorType::Regex,
                expected_value: None,

                error_node: Some(&#node_ident),
                    };
                    panic!("{}", ::assert_struct::__macro_support::format_errors_with_root(__PATTERN_TREE, vec![__error]));
                }
            }
        }
    } else {
        quote_spanned! {span=>
            {
                use ::assert_struct::Like;
                let re = ::regex::Regex::new(#pattern_str)
                    .expect(concat!("Invalid regex pattern: ", #pattern_str));
                if !(&#value_expr).like(&re) {
                    let __line = line!();
                    let __file = file!();
                    let __error = ::assert_struct::__macro_support::ErrorContext {
                        field_path: #field_path_str.to_string(),
                        pattern_str: #full_pattern_str.to_string(),
                        actual_value: format!("{:?}", &#value_expr),
                        line_number: __line,
                        file_name: __file,
                        error_type: ::assert_struct::__macro_support::ErrorType::Regex,
                expected_value: None,

                error_node: Some(&#node_ident),
                    };
                    panic!("{}", ::assert_struct::__macro_support::format_errors_with_root(__PATTERN_TREE, vec![__error]));
                }
            }
        }
    }
}

#[cfg(feature = "regex")]
/// Generate Like trait assertion with path tracking
fn generate_like_assertion_with_path(
    value_expr: &TokenStream,
    pattern_expr: &syn::Expr,
    is_ref: bool,
    path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    let field_path_str = path.join(".");
    let pattern_str = format!("=~ {}", quote! { #pattern_expr });

    if is_ref {
        quote! {
            {
                use ::assert_struct::Like;
                if !#value_expr.like(&#pattern_expr) {
                    let __line = line!();
                    let __file = file!();
                    let __error = ::assert_struct::__macro_support::ErrorContext {
                        field_path: #field_path_str.to_string(),
                        pattern_str: #pattern_str.to_string(),
                        actual_value: format!("{:?}", #value_expr),
                        line_number: __line,
                        file_name: __file,
                        error_type: ::assert_struct::__macro_support::ErrorType::Regex,
                expected_value: None,

                error_node: Some(&#node_ident),
                    };
                    panic!("{}", ::assert_struct::__macro_support::format_errors_with_root(__PATTERN_TREE, vec![__error]));
                }
            }
        }
    } else {
        quote! {
            {
                use ::assert_struct::Like;
                if !(&#value_expr).like(&#pattern_expr) {
                    let __line = line!();
                    let __file = file!();
                    let __error = ::assert_struct::__macro_support::ErrorContext {
                        field_path: #field_path_str.to_string(),
                        pattern_str: #pattern_str.to_string(),
                        actual_value: format!("{:?}", &#value_expr),
                        line_number: __line,
                        file_name: __file,
                        error_type: ::assert_struct::__macro_support::ErrorType::Regex,
                expected_value: None,

                error_node: Some(&#node_ident),
                    };
                    panic!("{}", ::assert_struct::__macro_support::format_errors_with_root(__PATTERN_TREE, vec![__error]));
                }
            }
        }
    }
}
