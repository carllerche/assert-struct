use crate::AssertStruct;
use crate::pattern::{
    ComparisonOp, FieldAssertion, FieldOperation, Pattern, PatternClosure, PatternComparison,
    PatternMap, PatternRange, PatternSimple, PatternSlice, PatternStruct, PatternTuple,
    PatternWildcard, TupleElement,
};
#[cfg(feature = "regex")]
use crate::pattern::{PatternLike, PatternRegex};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use std::collections::HashSet;
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
            #[allow(unused_assignments, clippy::neg_cmp_op_on_partial_ord, clippy::op_ref, clippy::zero_prefixed_literal, clippy::bool_comparison)]
            let __assert_struct_result = {
                use std::convert::AsRef;

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
        Pattern::Simple(PatternSimple { node_id, .. })
        | Pattern::Struct(PatternStruct { node_id, .. })
        | Pattern::Tuple(PatternTuple { node_id, .. })
        | Pattern::Slice(PatternSlice { node_id, .. })
        | Pattern::Comparison(PatternComparison { node_id, .. })
        | Pattern::Range(PatternRange { node_id, .. })
        | Pattern::Wildcard(PatternWildcard { node_id })
        | Pattern::Closure(PatternClosure { node_id, .. })
        | Pattern::Map(PatternMap { node_id, .. }) => *node_id,
        #[cfg(feature = "regex")]
        Pattern::Regex(PatternRegex { node_id, .. })
        | Pattern::Like(PatternLike { node_id, .. }) => *node_id,
    };
    Ident::new(&format!("__PATTERN_NODE_{}", node_id), Span::call_site())
}

/// Get the span for a pattern (if available)
fn get_pattern_span(pattern: &Pattern) -> Option<Span> {
    match pattern {
        Pattern::Simple(PatternSimple { expr, .. }) => Some(expr.span()),
        Pattern::Comparison(PatternComparison { expr, .. }) => Some(expr.span()),
        Pattern::Range(PatternRange { expr, .. }) => Some(expr.span()),
        #[cfg(feature = "regex")]
        Pattern::Regex(PatternRegex { span, .. }) => Some(*span),
        #[cfg(feature = "regex")]
        Pattern::Like(PatternLike { expr, .. }) => Some(expr.span()),
        Pattern::Struct(PatternStruct { path, .. }) => path.as_ref().map(|p| p.span()),
        Pattern::Tuple(PatternTuple { path, .. }) => path.as_ref().map(|p| p.span()),
        Pattern::Slice(PatternSlice { .. })
        | Pattern::Wildcard(PatternWildcard { .. })
        | Pattern::Map(PatternMap { .. }) => None,
        Pattern::Closure(PatternClosure { closure, .. }) => Some(closure.span()),
    }
}

/// Generate pattern nodes using the IDs already in patterns
fn generate_pattern_nodes(
    pattern: &Pattern,
    node_defs: &mut Vec<(usize, TokenStream)>,
) -> TokenStream {
    // Get the node_id from the pattern itself
    let node_id = match pattern {
        Pattern::Simple(PatternSimple { node_id, .. })
        | Pattern::Struct(PatternStruct { node_id, .. })
        | Pattern::Tuple(PatternTuple { node_id, .. })
        | Pattern::Slice(PatternSlice { node_id, .. })
        | Pattern::Comparison(PatternComparison { node_id, .. })
        | Pattern::Range(PatternRange { node_id, .. })
        | Pattern::Wildcard(PatternWildcard { node_id })
        | Pattern::Closure(PatternClosure { node_id, .. })
        | Pattern::Map(PatternMap { node_id, .. }) => *node_id,
        #[cfg(feature = "regex")]
        Pattern::Regex(PatternRegex { node_id, .. })
        | Pattern::Like(PatternLike { node_id, .. }) => *node_id,
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
        Pattern::Simple(PatternSimple { expr, .. }) => {
            let value_str = quote! { #expr }.to_string();
            quote! {
                ::assert_struct::__macro_support::PatternNode::Simple {
                    value: #value_str,
                }
            }
        }
        Pattern::Comparison(PatternComparison { op, expr, .. }) => {
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
        Pattern::Range(PatternRange { expr, .. }) => {
            let pattern_str = quote! { #expr }.to_string();
            quote! {
                ::assert_struct::__macro_support::PatternNode::Range {
                    pattern: #pattern_str,
                }
            }
        }
        #[cfg(feature = "regex")]
        Pattern::Regex(PatternRegex { pattern, .. }) => {
            let pattern_str = format!("r\"{}\"", pattern);
            quote! {
                ::assert_struct::__macro_support::PatternNode::Regex {
                    pattern: #pattern_str,
                }
            }
        }
        #[cfg(feature = "regex")]
        Pattern::Like(PatternLike { expr, .. }) => {
            let expr_str = quote! { #expr }.to_string();
            quote! {
                ::assert_struct::__macro_support::PatternNode::Like {
                    expr: #expr_str,
                }
            }
        }
        Pattern::Wildcard(PatternWildcard { .. }) => {
            quote! {
                ::assert_struct::__macro_support::PatternNode::Wildcard
            }
        }
        Pattern::Closure(PatternClosure { closure, .. }) => {
            let closure_str = quote! { #closure }.to_string();
            quote! {
                ::assert_struct::__macro_support::PatternNode::Closure {
                    closure: #closure_str,
                }
            }
        }
        Pattern::Tuple(PatternTuple { path, elements, .. }) => {
            let child_refs: Vec<TokenStream> = elements
                .iter()
                .map(|elem| {
                    let pattern = match elem {
                        TupleElement::Positional(elem) => &elem.pattern,
                        TupleElement::Indexed(elem) => &elem.pattern,
                    };
                    generate_pattern_nodes(pattern, node_defs)
                })
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
        Pattern::Slice(PatternSlice { elements, .. }) => {
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
        Pattern::Struct(PatternStruct {
            path, fields, rest, ..
        }) => {
            // Handle wildcard struct patterns (path is None)
            let name_str = if let Some(p) = path {
                quote! { #p }.to_string().replace(" :: ", "::")
            } else {
                "_".to_string() // Use "_" for wildcard struct patterns
            };

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
        Pattern::Map(PatternMap { entries, rest, .. }) => {
            let entry_refs: Vec<TokenStream> = entries
                .iter()
                .map(|(key, value)| {
                    let key_str = quote! { #key }.to_string();
                    let value_ref = generate_pattern_nodes(value, node_defs);
                    quote! {
                        (#key_str, &#value_ref)
                    }
                })
                .collect();

            if *rest {
                quote! {
                    ::assert_struct::__macro_support::PatternNode::Map {
                        entries: &[
                            #(#entry_refs,)*
                            ("..", &::assert_struct::__macro_support::PatternNode::Rest)
                        ],
                    }
                }
            } else {
                quote! {
                    ::assert_struct::__macro_support::PatternNode::Map {
                        entries: &[#(#entry_refs),*],
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
        Pattern::Simple(PatternSimple { expr: expected, .. }) => {
            generate_simple_assertion_with_collection(
                value_expr,
                expected,
                is_ref,
                path,
                &node_ident,
            )
        }
        Pattern::Struct(PatternStruct {
            path: struct_path,
            fields,
            rest,
            ..
        }) => generate_struct_match_assertion_with_collection(
            value_expr,
            struct_path, // Now passing &Option<syn::Path>
            fields,
            *rest,
            is_ref,
            path,
            &node_ident,
        ),
        Pattern::Comparison(PatternComparison {
            op, expr: expected, ..
        }) => generate_comparison_assertion_with_collection(
            value_expr,
            op,
            expected,
            is_ref,
            path,
            &pattern_str,
            &node_ident,
        ),
        Pattern::Tuple(PatternTuple {
            path: variant_path,
            elements,
            ..
        }) => {
            // Handle enum tuples with error collection
            if let Some(vpath) = variant_path {
                if elements.is_empty() {
                    // Unit variant like None - use collection version for proper error collection
                    generate_enum_tuple_assertion_with_collection(
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
                // Plain tuple - use collection version for proper error collection
                generate_plain_tuple_assertion_with_collection(
                    value_expr,
                    elements,
                    is_ref,
                    path,
                    &node_ident,
                )
            }
        }
        Pattern::Wildcard(PatternWildcard { .. }) => {
            // Wildcard patterns generate no assertions - they just verify the field exists
            // which is already handled by the struct/tuple destructuring
            quote! {}
        }
        Pattern::Range(PatternRange { expr: range, .. }) => {
            // Generate improved range assertion with error collection
            generate_range_assertion_with_collection(
                value_expr,
                range,
                is_ref,
                path,
                &pattern_str,
                &node_ident,
            )
        }
        Pattern::Slice(PatternSlice { elements, .. }) => {
            // Generate slice assertion with error collection
            generate_slice_assertion_with_collection(
                value_expr,
                elements,
                is_ref,
                path,
                &node_ident,
            )
        }
        #[cfg(feature = "regex")]
        Pattern::Regex(PatternRegex {
            pattern: regex_str,
            span,
            ..
        }) => {
            // Generate regex assertion with error collection
            generate_regex_assertion_with_collection(
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
        Pattern::Like(PatternLike {
            expr: pattern_expr, ..
        }) => {
            // Generate Like trait assertion with error collection
            generate_like_assertion_with_collection(
                value_expr,
                pattern_expr,
                is_ref,
                path,
                &node_ident,
            )
        }
        Pattern::Closure(PatternClosure { closure, .. }) => {
            // Generate closure assertion with error collection
            generate_closure_assertion_with_collection(
                value_expr,
                closure,
                is_ref,
                path,
                &node_ident,
            )
        }
        Pattern::Map(PatternMap { entries, rest, .. }) => {
            // Generate map assertion with error collection
            // Use span from first entry or default span if empty
            let map_span = entries
                .first()
                .map(|(key, _)| key.span())
                .unwrap_or_else(proc_macro2::Span::call_site);

            generate_map_assertion_with_collection(
                value_expr,
                entries,
                *rest,
                is_ref,
                path,
                &node_ident,
                map_span,
            )
        }
    }
}

/// Generate wildcard struct assertion using direct field access
fn generate_wildcard_struct_assertion_with_collection(
    value_expr: &TokenStream,
    fields: &Punctuated<FieldAssertion, Token![,]>,
    _is_ref: bool,
    field_path: &[String],
    _node_ident: &Ident,
) -> TokenStream {
    let field_assertions: Vec<_> = fields
        .iter()
        .map(|f| {
            let field_name = &f.field_name;
            let field_pattern = &f.pattern;
            let field_operations = &f.operations;

            // Build path for this field
            let mut new_path = field_path.to_vec();
            new_path.push(field_name.to_string());

            // Apply operations if any and determine reference level
            let (accessed_value, is_ref_after) = if let Some(ops) = field_operations {
                // For method calls, we need to access the field without taking a reference
                // since the method call will operate on the field directly
                let base_field_access = quote! { (#value_expr).#field_name };

                // Apply operations - pass false for in_ref_context since we're not taking a reference
                let expr = apply_field_operations(&base_field_access, ops, false);

                // Operations change the reference level based on their type
                let is_ref = field_operation_returns_reference(ops);
                (expr, is_ref)
            } else {
                // No operations - we need a reference to the field for comparison
                let field_access = quote! { &(#value_expr).#field_name };
                // field_access is already a reference, so is_ref = true
                (field_access, true)
            };

            // Recursively expand the pattern for this field
            generate_pattern_assertion_with_collection(
                &accessed_value,
                field_pattern,
                is_ref_after,
                &new_path,
            )
        })
        .collect();

    quote! {
        #(#field_assertions)*
    }
}

/// Generate struct assertion with error collection for multiple field failures
fn generate_struct_match_assertion_with_collection(
    value_expr: &TokenStream,
    struct_path: &Option<syn::Path>,
    fields: &Punctuated<FieldAssertion, Token![,]>,
    rest: bool,
    is_ref: bool,
    field_path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    // If struct_path is None, it's a wildcard pattern - use field access
    if struct_path.is_none() {
        return generate_wildcard_struct_assertion_with_collection(
            value_expr, fields, is_ref, field_path, node_ident,
        );
    }

    let struct_path = struct_path.as_ref().unwrap();

    // For nested field access, we need to collect unique field names only
    // If we have middle.inner.value and middle.count, we only want "middle" once
    let mut unique_field_names = HashSet::new();
    let field_names: Vec<_> = fields
        .iter()
        .filter_map(|f| {
            if unique_field_names.insert(f.field_name.clone()) {
                Some(&f.field_name)
            } else {
                None
            }
        })
        .collect();

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
            let field_operations = &f.operations;

            // Build path for this field - include operations if present
            let mut new_path = field_path.to_vec();
            let field_path_str = if let Some(ops) = field_operations {
                // Include operation in path for better error messages
                generate_field_operation_path(field_name.to_string(), ops)
            } else {
                field_name.to_string()
            };
            new_path.push(field_path_str);

            // Generate the value expression with operations applied
            let (value_expr, is_ref_after_operations) = if let Some(ops) = field_operations {
                // We're always in a reference context for struct destructuring (match &value)
                let expr = apply_field_operations(&quote! { #field_name }, ops, true);
                // Operations change the reference level based on their type
                let is_ref = field_operation_returns_reference(ops);
                (expr, is_ref)
            } else {
                (quote! { #field_name }, true)
            };

            // Generate assertion with appropriate reference handling
            let assertion = generate_pattern_assertion_with_collection(
                &value_expr,
                field_pattern,
                is_ref_after_operations,
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

    let span = struct_path.span();
    if is_ref {
        quote_spanned! {span=>
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
        quote_spanned! {span=>
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

/// Generate a readable path string for a field operation for error messages
fn generate_field_operation_path(base_field: String, operation: &FieldOperation) -> String {
    match operation {
        FieldOperation::Deref { count, .. } => {
            let stars = "*".repeat(*count);
            format!("{}{}", stars, base_field)
        }
        FieldOperation::Method { name, .. } => {
            format!("{}.{}()", base_field, name)
        }
        FieldOperation::Await { .. } => {
            format!("{}.await", base_field)
        }
        FieldOperation::Nested { fields, .. } => {
            let nested = fields
                .iter()
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join(".");
            format!("{}.{}", base_field, nested)
        }
        FieldOperation::Index { index, .. } => {
            format!("{}[{}]", base_field, quote! { #index })
        }
        FieldOperation::Combined {
            deref_count,
            operation,
            ..
        } => {
            let stars = "*".repeat(*deref_count);
            let base_with_deref = format!("{}{}", stars, base_field);
            generate_field_operation_path(base_with_deref, operation)
        }
        FieldOperation::Chained { operations, .. } => {
            let mut result = base_field;
            for op in operations {
                result = generate_field_operation_path(result, op);
            }
            result
        }
    }
}

/// Apply field operations to a value expression
/// This generates the appropriate dereferencing, method calls, nested field access, index operations, or await
///
/// # Parameters
/// - `base_expr`: The base expression to apply operations to
/// - `operation`: The field operation to apply
/// - `in_ref_context`: Whether we're in a reference context (from destructuring `&value`)
fn apply_field_operations(
    base_expr: &TokenStream,
    operation: &FieldOperation,
    in_ref_context: bool,
) -> TokenStream {
    match operation {
        FieldOperation::Deref { count, span } => {
            let mut expr = base_expr.clone();
            // In reference context, we need one extra dereference
            let total_count = if in_ref_context { count + 1 } else { *count };
            for _ in 0..total_count {
                expr = quote_spanned! { *span=> *#expr };
            }
            expr
        }
        FieldOperation::Method { name, args, span } => {
            if args.is_empty() {
                quote_spanned! { *span=> #base_expr.#name() }
            } else {
                quote_spanned! { *span=> #base_expr.#name(#(#args),*) }
            }
        }
        FieldOperation::Await { span } => {
            quote_spanned! { *span=> #base_expr.await }
        }
        FieldOperation::Nested { fields, span } => {
            let mut expr = base_expr.clone();
            for field in fields {
                expr = quote_spanned! { *span=> #expr.#field };
            }
            expr
        }
        FieldOperation::Index { index, span } => {
            quote_spanned! { *span=> #base_expr[#index] }
        }
        FieldOperation::Combined {
            deref_count,
            operation,
            span,
        } => {
            // First apply dereferencing with reference context awareness
            let mut expr = base_expr.clone();
            let total_count = if in_ref_context {
                deref_count + 1
            } else {
                *deref_count
            };
            for _ in 0..total_count {
                expr = quote_spanned! { *span=> *#expr };
            }
            // Then apply the nested operation (no longer in ref context after deref)
            apply_field_operations(&expr, operation, false)
        }
        FieldOperation::Chained { operations, .. } => {
            let mut expr = base_expr.clone();
            for op in operations {
                expr = apply_field_operations(&expr, op, false);
            }
            expr
        }
    }
}

/// Helper function to determine if a field operation returns a reference
fn field_operation_returns_reference(operation: &FieldOperation) -> bool {
    match operation {
        FieldOperation::Deref { .. } => false, // Dereferencing removes reference level
        FieldOperation::Method { .. } => false, // Method calls return owned values
        FieldOperation::Await { .. } => false, // Await returns owned values
        FieldOperation::Nested { .. } => false, // Nested field access auto-derefs to get field value
        FieldOperation::Index { .. } => true,   // Index operations return references to elements
        FieldOperation::Combined { .. } => false, // Combined with deref also removes reference level
        FieldOperation::Chained { operations, .. } => {
            // For chained operations, the reference level is determined by the last operation
            operations
                .last()
                .map(field_operation_returns_reference)
                .unwrap_or(false)
        }
    }
}

/// Helper function to process tuple elements and generate match patterns and assertions.
///
/// This function handles the common pattern of iterating through elements and:
/// - Creating `_` patterns for wildcards (which need no assertions)
/// - Creating named bindings for other patterns and generating their assertions
/// - Handling field operations like dereferencing
///
/// # Parameters
/// - `elements`: The tuple elements to process (can be positional or indexed)
/// - `prefix`: Prefix for generated binding names (e.g., "__elem_", "__tuple_elem_")
/// - `is_ref`: Whether the bindings are already references
/// - `field_path`: Path components for error messages
///
/// # Returns
/// A tuple of (match_patterns, assertions) where:
/// - `match_patterns`: TokenStreams for use in match arms or destructuring
/// - `assertions`: TokenStreams for the generated assertion code
fn process_tuple_elements(
    elements: &[TupleElement],
    prefix: &str,
    is_ref: bool,
    field_path: &[String],
) -> (Vec<TokenStream>, Vec<TokenStream>) {
    let mut match_patterns = Vec::new();
    let mut assertions = Vec::new();

    for (i, tuple_element) in elements.iter().enumerate() {
        let (pattern, operations) = match tuple_element {
            TupleElement::Positional(elem) => (&elem.pattern, &None),
            TupleElement::Indexed(elem) => (&elem.pattern, &elem.operations),
        };

        match pattern {
            Pattern::Wildcard(PatternWildcard { .. }) => {
                // Wildcard patterns use `_` in the match pattern.
                // No assertion is generated because wildcards only verify
                // that the element exists (handled by the match itself).
                match_patterns.push(quote! { _ });
            }
            _ => {
                // Non-wildcard patterns need a binding and assertion
                let name = quote::format_ident!("{}{}", prefix, i);
                match_patterns.push(quote! { #name });

                // Build path for error messages - include operations if present
                let mut elem_path = field_path.to_vec();
                let elem_path_str = if let Some(ops) = operations {
                    // Include operation in path for better error messages
                    generate_field_operation_path(i.to_string(), ops)
                } else {
                    i.to_string()
                };
                elem_path.push(elem_path_str);

                // Generate the value expression with operations applied
                let value_expr = if let Some(ops) = operations {
                    // Tuple elements are in reference context when destructured
                    apply_field_operations(&quote! { #name }, ops, true)
                } else {
                    quote! { #name }
                };

                // Apply the same reference level logic as for struct fields
                let is_ref_after_operations = if let Some(ops) = operations {
                    field_operation_returns_reference(ops)
                } else {
                    is_ref
                };

                // Generate assertion with error collection
                let assertion = generate_pattern_assertion_with_collection(
                    &value_expr,
                    pattern,
                    is_ref_after_operations,
                    &elem_path,
                );
                assertions.push(assertion);
            }
        }
    }

    (match_patterns, assertions)
}

/// Generate assertion for plain tuples with error collection.
/// Uses match expressions for consistency with enum tuple handling.
fn generate_plain_tuple_assertion_with_collection(
    value_expr: &TokenStream,
    elements: &[TupleElement],
    is_ref: bool,
    field_path: &[String],
    _node_ident: &Ident,
) -> TokenStream {
    // Use helper to process elements with collection strategy
    let (match_patterns, element_assertions) =
        process_tuple_elements(elements, "__tuple_elem_", true, field_path);

    // Use match expression for consistency with enum tuples
    // The unreachable _ arm is acceptable for plain tuples
    if is_ref {
        quote! {
            #[allow(unreachable_patterns)]
            match #value_expr {
                (#(#match_patterns),*) => {
                    #(#element_assertions)*
                },
                _ => unreachable!("Plain tuple match should always succeed"),
            }
        }
    } else {
        quote! {
            #[allow(unreachable_patterns)]
            match &#value_expr {
                (#(#match_patterns),*) => {
                    #(#element_assertions)*
                },
                _ => unreachable!("Plain tuple match should always succeed"),
            }
        }
    }
}

// Check if a path refers to Option::Some

/// Convert a pattern to its string representation for error messages
fn pattern_to_string(pattern: &Pattern) -> String {
    match pattern {
        Pattern::Simple(PatternSimple { expr, .. }) => quote! { #expr }.to_string(),
        Pattern::Comparison(PatternComparison { op, expr, .. }) => {
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
        Pattern::Range(PatternRange { expr: range, .. }) => quote! { #range }.to_string(),
        #[cfg(feature = "regex")]
        Pattern::Regex(PatternRegex { pattern, .. }) => format!("=~ r\"{}\"", pattern),
        #[cfg(feature = "regex")]
        Pattern::Like(PatternLike { expr, .. }) => format!("=~ {}", quote! { #expr }),
        Pattern::Wildcard(PatternWildcard { .. }) => "_".to_string(),
        Pattern::Closure(PatternClosure { closure, .. }) => quote! { #closure }.to_string(),
        Pattern::Struct(PatternStruct { path, .. }) => {
            if let Some(p) = path {
                quote! { #p { .. } }.to_string()
            } else {
                "_ { .. }".to_string()
            }
        }
        Pattern::Tuple(PatternTuple { path, elements, .. }) => {
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
        Pattern::Map(PatternMap { entries, rest, .. }) => {
            if *rest {
                format!("#{{ {} entries, .. }}", entries.len())
            } else {
                format!("#{{ {} entries }}", entries.len())
            }
        }
        Pattern::Slice(PatternSlice { elements, .. }) => format!("[{} elements]", elements.len()),
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

    // Check if this is an index operation by looking at the path
    // Exclude slice patterns which start with [
    let is_index_operation = path
        .iter()
        .any(|segment| segment.contains("[") && !segment.starts_with("["));

    // Adjust for reference level
    let actual_expr = if is_ref {
        quote! { #value_expr }
    } else {
        quote! { &#value_expr }
    };

    let span = expected.span();
    let comparison = if is_index_operation {
        // For index operations, avoid references on both sides
        match op {
            ComparisonOp::Less => quote_spanned! {span=> #value_expr < #expected },
            ComparisonOp::LessEqual => quote_spanned! {span=> #value_expr <= #expected },
            ComparisonOp::Greater => quote_spanned! {span=> #value_expr > #expected },
            ComparisonOp::GreaterEqual => quote_spanned! {span=> #value_expr >= #expected },
            ComparisonOp::Equal => quote_spanned! {span=> #value_expr == #expected },
            ComparisonOp::NotEqual => quote_spanned! {span=> #value_expr != #expected },
        }
    } else if is_ref {
        match op {
            ComparisonOp::Less => quote_spanned! {span=> #value_expr < &(#expected) },
            ComparisonOp::LessEqual => quote_spanned! {span=> #value_expr <= &(#expected) },
            ComparisonOp::Greater => quote_spanned! {span=> #value_expr > &(#expected) },
            ComparisonOp::GreaterEqual => quote_spanned! {span=> #value_expr >= &(#expected) },
            ComparisonOp::Equal => quote_spanned! {span=> #value_expr == &(#expected) },
            ComparisonOp::NotEqual => quote_spanned! {span=> #value_expr != &(#expected) },
        }
    } else {
        match op {
            ComparisonOp::Less => quote_spanned! {span=> &#value_expr < &(#expected) },
            ComparisonOp::LessEqual => quote_spanned! {span=> &#value_expr <= &(#expected) },
            ComparisonOp::Greater => quote_spanned! {span=> &#value_expr > &(#expected) },
            ComparisonOp::GreaterEqual => quote_spanned! {span=> &#value_expr >= &(#expected) },
            ComparisonOp::Equal => quote_spanned! {span=> &#value_expr == &(#expected) },
            ComparisonOp::NotEqual => quote_spanned! {span=> &#value_expr != &(#expected) },
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

    quote_spanned! {span=>
        #[allow(clippy::nonminimal_bool)]
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

/// Generate assertion for enum tuple variants with error collection
fn generate_enum_tuple_assertion_with_collection(
    value_expr: &TokenStream,
    variant_path: &syn::Path,
    elements: &[TupleElement],
    is_ref: bool,
    field_path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    let field_path_str = field_path.join(".");
    let span = variant_path.span();

    // Special handling for unit variants (empty elements)
    if elements.is_empty() {
        // Unit variants don't have parentheses
        if is_ref {
            quote_spanned! {span=>
                match #value_expr {
                    #variant_path => {},
                    _ => {
                        let __line = line!();
                        let __file = file!();

                        let __error = ::assert_struct::__macro_support::ErrorContext {
                            field_path: #field_path_str.to_string(),
                            pattern_str: stringify!(#variant_path).to_string(),
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
            quote_spanned! {span=>
                match &#value_expr {
                    #variant_path => {},
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
    } else {
        // Tuple variants with elements
        // Extract variant name from path for better error messages
        let variant_name = if let Some(segment) = variant_path.segments.last() {
            segment.ident.to_string()
        } else {
            "variant".to_string()
        };

        // Build path with variant name for single-element tuples
        let mut base_path = field_path.to_vec();
        let use_variant_name = elements.len() == 1;

        // Use helper to process elements with appropriate path
        let (match_patterns, element_assertions) = if use_variant_name {
            base_path.push(variant_name);
            process_tuple_elements(elements, "__elem_", true, &base_path)
        } else {
            process_tuple_elements(elements, "__elem_", true, field_path)
        };

        if is_ref {
            quote_spanned! {span=>
                match #value_expr {
                    #variant_path(#(#match_patterns),*) => {
                        #(#element_assertions)*
                    },
                    _ => {
                        let __line = line!();
                        let __file = file!();

                        let __error = ::assert_struct::__macro_support::ErrorContext {
                            field_path: #field_path_str.to_string(),
                            pattern_str: stringify!(#variant_path).to_string(),
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
            quote_spanned! {span=>
                match &#value_expr {
                    #variant_path(#(#match_patterns),*) => {
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
}

/// Generate range assertion with error collection
fn generate_range_assertion_with_collection(
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

                __errors.push(__error);
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
    let field_path_str = path.join(".");
    let expected_str = quote! { #expected }.to_string();

    let actual = match expected {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(_),
            ..
        }) => {
            quote!((#value_expr).as_ref())
        }
        _ => quote!(#value_expr),
    };

    // Check if this is an index operation by looking at the path
    // Exclude slice patterns which start with [
    let is_index_operation = path
        .iter()
        .any(|segment| segment.contains("[") && !segment.starts_with("["));

    let span = expected.span();
    if is_index_operation {
        // For index operations, avoid references on both sides to fix type inference
        quote_spanned! {span=>
            if !matches!(#actual, #expected) {
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
    } else if is_ref {
        quote_spanned! {span=>
            if !matches!(#actual, #expected) {
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
            if !matches!(#actual, #expected) {
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

/// Generate slice assertion with error collection
fn generate_slice_assertion_with_collection(
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
            Pattern::Range(PatternRange {
                expr: syn::Expr::Range(r),
                ..
            }) if r.start.is_none() && r.end.is_none() => {
                // RangeFull (..) in slice context is a rest pattern
                pattern_parts.push(quote! { .. });
            }
            Pattern::Wildcard(PatternWildcard { .. }) => {
                // Wildcard pattern matches any single element without binding
                pattern_parts.push(quote! { _ });
            }
            _ => {
                let binding = quote::format_ident!("__elem_{}", i);
                pattern_parts.push(quote! { #binding });

                // Build path for this slice element
                let mut elem_path = field_path.to_vec();
                elem_path.push(format!("[{}]", i));

                let assertion = generate_pattern_assertion_with_collection(
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
                __errors.push(__error);
            }
        }
    }
}

#[cfg(feature = "regex")]
/// Generate regex assertion with error collection
fn generate_regex_assertion_with_collection(
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
                let re = ::assert_struct::__macro_support::Regex::new(#pattern_str)
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
                    __errors.push(__error);
                }
            }
        }
    } else {
        quote_spanned! {span=>
            {
                use ::assert_struct::Like;
                let re = ::assert_struct::__macro_support::Regex::new(#pattern_str)
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
                    __errors.push(__error);
                }
            }
        }
    }
}

#[cfg(feature = "regex")]
/// Generate Like trait assertion with error collection
fn generate_like_assertion_with_collection(
    value_expr: &TokenStream,
    pattern_expr: &syn::Expr,
    is_ref: bool,
    path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    let field_path_str = path.join(".");
    let pattern_str = format!("=~ {}", quote! { #pattern_expr });

    let span = pattern_expr.span();
    if is_ref {
        quote_spanned! {span=>
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
                    __errors.push(__error);
                }
            }
        }
    } else {
        quote_spanned! {span=>
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
                    __errors.push(__error);
                }
            }
        }
    }
}

/// Generate closure assertion with error collection
fn generate_closure_assertion_with_collection(
    value_expr: &TokenStream,
    closure: &syn::ExprClosure,
    is_ref: bool,
    path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    let field_path_str = path.join(".");
    let closure_str = quote! { #closure }.to_string();

    // Adjust for reference level - closures receive the actual value
    let actual_expr = if is_ref {
        quote! { #value_expr }
    } else {
        quote! { &#value_expr }
    };

    let span = closure.span();
    quote_spanned! {span=>
        {
            if !::assert_struct::__macro_support::check_closure_condition(#actual_expr, #closure) {
                let __line = line!();
                let __file = file!();
                let __error = ::assert_struct::__macro_support::ErrorContext {
                    field_path: #field_path_str.to_string(),
                    pattern_str: #closure_str.to_string(),
                    actual_value: format!("{:?}", #actual_expr),
                    line_number: __line,
                    file_name: __file,
                    error_type: ::assert_struct::__macro_support::ErrorType::Closure,
                    expected_value: None,
                    error_node: Some(&#node_ident),
                };
                __errors.push(__error);
            }
        }
    }
}

/// Generate map assertion with error collection using duck typing
/// Assumes map types have len() -> usize and get(&K) -> Option<&V> methods
fn generate_map_assertion_with_collection(
    value_expr: &TokenStream,
    entries: &[(syn::Expr, Pattern)],
    rest: bool,
    _is_ref: bool,
    path: &[String],
    node_ident: &Ident,
    map_span: proc_macro2::Span,
) -> TokenStream {
    let field_path_str = path.join(".");

    // Generate length check assertion for exact matching (when no rest pattern)
    let len_check = if !rest {
        let expected_len = entries.len();
        quote_spanned! {map_span=>
            // Check exact length for maps without rest pattern
            if (#value_expr).len() != #expected_len {
                let __line = line!();
                let __file = file!();
                let __error = ::assert_struct::__macro_support::ErrorContext {
                    field_path: #field_path_str.to_string(),
                    pattern_str: format!("#{{ {} entries }}", #expected_len),
                    actual_value: format!("map with {} entries", (#value_expr).len()),
                    line_number: __line,
                    file_name: __file,
                    error_type: ::assert_struct::__macro_support::ErrorType::Value,
                    expected_value: Some(format!("{} entries", #expected_len)),
                    error_node: Some(&#node_ident),
                };
                __errors.push(__error);
            }
        }
    } else {
        quote! {}
    };

    // Generate key-value assertions
    let key_value_assertions: Vec<TokenStream> = entries
        .iter()
        .map(|(key, value_pattern)| {
            let key_str = quote! { #key }.to_string();

            // Build path for this key
            let mut key_path = path.to_vec();
            key_path.push(key_str.clone());

            let span = key.span();
            let pattern_assertion = generate_pattern_assertion_with_collection(
                &quote! { __map_value },
                value_pattern,
                true, // map.get() returns Option<&V>, so we have a reference
                &key_path,
            );

            // Handle different key types for duck typing
            let get_expr = if matches!(
                key,
                syn::Expr::Lit(syn::ExprLit {
                    lit: syn::Lit::Str(_),
                    ..
                })
            ) {
                // For string literals, convert to String to match HashMap<String, V>
                quote_spanned! {span=> (#value_expr).get(&(#key).to_string()) }
            } else {
                // For other expressions, try as-is
                quote_spanned! {span=> (#value_expr).get(&#key) }
            };

            quote_spanned! {span=>
                // Check if key exists and apply pattern to the value
                match #get_expr {
                    Some(__map_value) => {
                        // Apply pattern assertion to the value
                        #pattern_assertion
                    }
                    None => {
                        let __line = line!();
                        let __file = file!();
                        let __error = ::assert_struct::__macro_support::ErrorContext {
                            field_path: #field_path_str.to_string(),
                            pattern_str: format!("key: {}", #key_str),
                            actual_value: "missing key".to_string(),
                            line_number: __line,
                            file_name: __file,
                            error_type: ::assert_struct::__macro_support::ErrorType::Value,
                            expected_value: Some(format!("key present: {}", #key_str)),
                            error_node: Some(&#node_ident),
                        };
                        __errors.push(__error);
                    }
                }
            }
        })
        .collect();

    quote! {
        #len_check
        #(#key_value_assertions)*
    }
}
