use crate::AssertStruct;
use crate::pattern::{
    ComparisonOp, FieldAssertion, FieldOperation, Pattern, PatternClosure, PatternComparison,
    PatternEnum, PatternMap, PatternRange, PatternSimple, PatternSlice, PatternString,
    PatternStruct, PatternTuple, PatternWildcard, TupleElement,
};
#[cfg(feature = "regex")]
use crate::pattern::{PatternLike, PatternRegex};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use std::collections::HashSet;
use syn::{Token, punctuated::Punctuated, spanned::Spanned};

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
            #[allow(unused_assignments, clippy::neg_cmp_op_on_partial_ord, clippy::op_ref, clippy::zero_prefixed_literal, clippy::bool_comparison, clippy::redundant_pattern_matching, clippy::useless_asref)]
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
        | Pattern::String(PatternString { node_id, .. })
        | Pattern::Struct(PatternStruct { node_id, .. })
        | Pattern::Enum(PatternEnum { node_id, .. })
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
        Pattern::String(PatternString { lit, .. }) => Some(lit.span()),
        Pattern::Comparison(PatternComparison { expr, .. }) => Some(expr.span()),
        Pattern::Range(PatternRange { expr, .. }) => Some(expr.span()),
        #[cfg(feature = "regex")]
        Pattern::Regex(PatternRegex { span, .. }) => Some(*span),
        #[cfg(feature = "regex")]
        Pattern::Like(PatternLike { expr, .. }) => Some(expr.span()),
        Pattern::Struct(PatternStruct { path, .. }) => path.as_ref().map(|p| p.span()),
        Pattern::Enum(PatternEnum { path, .. }) => Some(path.span()),
        Pattern::Tuple(PatternTuple { .. })
        | Pattern::Slice(PatternSlice { .. })
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
        | Pattern::String(PatternString { node_id, .. })
        | Pattern::Struct(PatternStruct { node_id, .. })
        | Pattern::Enum(PatternEnum { node_id, .. })
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
        Pattern::String(PatternString { lit, .. }) => {
            let value_str = format!("\"{}\"", lit.value());
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
        Pattern::Enum(PatternEnum { path, elements, .. }) => {
            let child_refs: Vec<TokenStream> = elements
                .iter()
                .map(|elem| {
                    let pattern = match elem {
                        TupleElement::Positional(pattern) => pattern,
                        TupleElement::Indexed(boxed_elem) => &boxed_elem.pattern,
                    };
                    generate_pattern_nodes(pattern, node_defs)
                })
                .collect();

            let path_str = quote!(#path).to_string().replace(" :: ", "::");
            let args = if elements.is_empty() {
                quote!(None)
            } else {
                quote!(Some(&[#(&#child_refs),*]))
            };

            quote! {
                ::assert_struct::__macro_support::PatternNode::EnumVariant {
                    path: #path_str,
                    args: #args,
                }
            }
        }
        Pattern::Tuple(PatternTuple { elements, .. }) => {
            let child_refs: Vec<TokenStream> = elements
                .iter()
                .map(|elem| {
                    let pattern = match elem {
                        TupleElement::Positional(pattern) => pattern,
                        TupleElement::Indexed(boxed_elem) => &boxed_elem.pattern,
                    };
                    generate_pattern_nodes(pattern, node_defs)
                })
                .collect();

            quote! {
                ::assert_struct::__macro_support::PatternNode::Tuple {
                    items: &[#(&#child_refs),*],
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
                    let field_name = field.operations.root_field_name().to_string();
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
    // Get the node identifier for this pattern
    let node_ident = get_pattern_node_ident(pattern);

    match pattern {
        Pattern::Simple(simple_pattern) => {
            generate_simple_assertion_with_collection(value_expr, simple_pattern, path, &node_ident)
        }
        Pattern::String(string_pattern) => {
            generate_string_assertion_with_collection(value_expr, string_pattern, path, &node_ident)
        }
        Pattern::Struct(struct_pattern) => generate_struct_match_assertion_with_collection(
            value_expr,
            struct_pattern,
            path,
            &node_ident,
        ),
        Pattern::Comparison(comparison_pattern) => generate_comparison_assertion_with_collection(
            value_expr,
            comparison_pattern,
            path,
            &node_ident,
        ),
        Pattern::Enum(enum_pattern) => {
            // Enum tuple variant - use collection version
            generate_enum_tuple_assertion_with_collection(
                value_expr,
                enum_pattern,
                path,
                &node_ident,
            )
        }
        Pattern::Tuple(tuple_pattern) => {
            // Plain tuple - use collection version for proper error collection
            generate_plain_tuple_assertion_with_collection(
                value_expr,
                tuple_pattern,
                path,
                &node_ident,
            )
        }
        Pattern::Wildcard(PatternWildcard { .. }) => {
            // Wildcard patterns generate no assertions - they just verify the field exists
            // which is already handled by the struct/tuple destructuring
            quote! {}
        }
        Pattern::Range(range_pattern) => {
            // Generate improved range assertion with error collection
            generate_range_assertion_with_collection(value_expr, range_pattern, path, &node_ident)
        }
        Pattern::Slice(slice_pattern) => {
            // Generate slice assertion with error collection
            generate_slice_assertion_with_collection(value_expr, slice_pattern, path, &node_ident)
        }
        #[cfg(feature = "regex")]
        Pattern::Regex(regex_pattern) => {
            // Generate regex assertion with error collection
            generate_regex_assertion_with_collection(value_expr, regex_pattern, path, &node_ident)
        }
        #[cfg(feature = "regex")]
        Pattern::Like(like_pattern) => {
            // Generate Like trait assertion with error collection
            generate_like_assertion_with_collection(
                value_expr,
                like_pattern,
                is_ref,
                path,
                &node_ident,
            )
        }
        Pattern::Closure(closure_pattern) => {
            // Generate closure assertion with error collection
            generate_closure_assertion_with_collection(
                value_expr,
                closure_pattern,
                path,
                &node_ident,
            )
        }
        Pattern::Map(map_pattern) => {
            // Generate map assertion with error collection
            generate_map_assertion_with_collection(
                value_expr,
                map_pattern,
                is_ref,
                path,
                &node_ident,
            )
        }
    }
}

/// Generate wildcard struct assertion using direct field access
fn generate_wildcard_struct_assertion_with_collection(
    value_expr: &TokenStream,
    fields: &Punctuated<FieldAssertion, Token![,]>,
    field_path: &[String],
    _node_ident: &Ident,
) -> TokenStream {
    let field_assertions: Vec<_> = fields
        .iter()
        .map(|f| {
            let field_name = f.operations.root_field_name();
            let field_pattern = &f.pattern;
            let field_operations = &f.operations;

            // Build path for this field - include full operations for error messages
            let mut new_path = field_path.to_vec();
            let field_path_str = if let Some(tail_ops) = field_operations.tail_operations() {
                generate_field_operation_path(field_name.to_string(), &tail_ops)
            } else {
                field_name.to_string()
            };
            new_path.push(field_path_str);

            // Access the field and apply tail operations
            let base_field_access = quote! { (#value_expr).#field_name };

            let (expr, is_ref_after) = if let Some(tail_ops) = field_operations.tail_operations() {
                // Apply remaining operations after the field access
                let expr = apply_field_operations(&base_field_access, &tail_ops);
                let is_ref = field_operation_returns_reference(&tail_ops);
                (expr, is_ref)
            } else {
                // No additional operations, take a reference to the field for comparison
                (quote! { &#base_field_access }, true)
            };

            // Recursively expand the pattern for this field
            generate_pattern_assertion_with_collection(
                &expr,
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
    struct_pattern: &PatternStruct,
    field_path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    let struct_path = &struct_pattern.path;
    let fields = &struct_pattern.fields;
    let rest = struct_pattern.rest;

    // If struct_path is None, it's a wildcard pattern - use field access
    let Some(struct_path) = struct_path.as_ref() else {
        return generate_wildcard_struct_assertion_with_collection(
            value_expr, fields, field_path, node_ident,
        );
    };

    // For nested field access, we need to collect unique field names only
    // If we have middle.inner.value and middle.count, we only want "middle" once
    let mut unique_field_names = HashSet::new();
    let field_names: Vec<_> = fields
        .iter()
        .filter_map(|f| {
            let field_name = f.operations.root_field_name();
            if unique_field_names.insert(field_name.clone()) {
                Some(field_name)
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
            let field_name = f.operations.root_field_name();

            // Expand the FieldAssertion starting from the bound field name
            let assertion = expand_field_assertion(&quote! { #field_name }, f, field_path);

            // Wrap the assertion with the span of the field pattern if available
            if let Some(span) = get_pattern_span(&f.pattern) {
                quote_spanned! {span=> #assertion }
            } else {
                assertion
            }
        })
        .collect();

    let span = struct_path.span();
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
        FieldOperation::NamedField { name, .. } => {
            format!("{}.{}", base_field, name)
        }
        FieldOperation::UnnamedField { index, .. } => {
            format!("{}.{}", base_field, index)
        }
        FieldOperation::Index { index, .. } => {
            format!("{}[{}]", base_field, quote! { #index })
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

/// Expand a FieldAssertion starting from a bound field name
///
/// This assumes the base field (root field name from the FieldOperation) is already bound
/// to `base`. It applies any tail operations and generates the pattern assertion.
///
/// # Parameters
/// - `base`: Expression for the bound field (e.g., `field_name` or `__tuple_elem_0`)
/// - `field_assertion`: The FieldAssertion to expand
/// - `is_ref_context`: Whether we're in a reference context (from destructuring)
/// - `base_field_path`: The field path up to the base field
fn expand_field_assertion(
    base: &TokenStream,
    field_assertion: &FieldAssertion,
    base_field_path: &[String],
) -> TokenStream {
    let field_operations = &field_assertion.operations;
    let field_pattern = &field_assertion.pattern;

    // Build the field path for error messages
    let mut new_path = base_field_path.to_vec();
    let root_field_name = field_operations.root_field_name();
    let field_path_str = if let Some(tail_ops) = field_operations.tail_operations() {
        generate_field_operation_path(root_field_name.to_string(), &tail_ops)
    } else {
        root_field_name.to_string()
    };
    new_path.push(field_path_str);

    // Apply tail operations and determine final reference context
    let expr = if let Some(tail_ops) = field_operations.tail_operations() {
        apply_field_operations(base, &tail_ops)
    } else {
        base.clone()
    };

    // Generate the pattern assertion
    generate_pattern_assertion_with_collection(&expr, field_pattern, true, &new_path)
}

/// Apply field operations to a value expression
/// This generates the appropriate dereferencing, method calls, nested field access, index operations, or await
///
/// # Parameters
/// - `base_expr`: The base expression to apply operations to
/// - `operation`: The field operation to apply
/// - `in_ref_context`: Whether we're in a reference context (from destructuring `&value`)
fn apply_field_operations(base_expr: &TokenStream, operation: &FieldOperation) -> TokenStream {
    match operation {
        FieldOperation::Deref { count, span } => {
            let mut expr = base_expr.clone();
            // In reference context, we need one extra dereference
            let total_count = count + 1;
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
        FieldOperation::NamedField { name, span } => {
            quote_spanned! { *span=> #base_expr.#name }
        }
        FieldOperation::UnnamedField { index, span } => {
            let idx = syn::Index::from(*index);
            quote_spanned! { *span=> #base_expr.#idx }
        }
        FieldOperation::Index { index, span } => {
            quote_spanned! { *span=> #base_expr[#index] }
        }
        FieldOperation::Chained { operations, .. } => {
            let mut expr = base_expr.clone();
            for op in operations {
                expr = apply_field_operations(&expr, op);
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
        FieldOperation::NamedField { .. } => false, // Field access auto-derefs to get field value
        FieldOperation::UnnamedField { .. } => false, // Tuple field access auto-derefs to get field value
        FieldOperation::Index { .. } => true, // Index operations return references to elements
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
        match tuple_element {
            TupleElement::Positional(pattern) => {
                match pattern {
                    Pattern::Wildcard(PatternWildcard { .. }) => {
                        // Wildcard patterns use `_` in the match pattern
                        match_patterns.push(quote! { _ });
                    }
                    _ => {
                        // Non-wildcard patterns need a binding and assertion
                        let name = quote::format_ident!("{}{}", prefix, i);
                        match_patterns.push(quote! { #name });

                        // Build path for error messages
                        let mut elem_path = field_path.to_vec();
                        elem_path.push(i.to_string());

                        // Generate assertion with error collection
                        let assertion = generate_pattern_assertion_with_collection(
                            &quote! { #name },
                            pattern,
                            is_ref,
                            &elem_path,
                        );
                        assertions.push(assertion);
                    }
                }
            }
            TupleElement::Indexed(boxed_elem) => {
                let pattern = &boxed_elem.pattern;

                match pattern {
                    Pattern::Wildcard(PatternWildcard { .. }) => {
                        // Wildcard patterns use `_` in the match pattern
                        match_patterns.push(quote! { _ });
                    }
                    _ => {
                        // Non-wildcard patterns need a binding and assertion
                        let name = quote::format_ident!("{}{}", prefix, i);
                        match_patterns.push(quote! { #name });

                        // Expand the indexed FieldAssertion starting from the bound element name
                        let assertion =
                            expand_field_assertion(&quote! { #name }, boxed_elem, field_path);
                        assertions.push(assertion);
                    }
                }
            }
        }
    }

    (match_patterns, assertions)
}

/// Generate assertion for plain tuples with error collection.
/// Uses match expressions for consistency with enum tuple handling.
fn generate_plain_tuple_assertion_with_collection(
    value_expr: &TokenStream,
    pattern: &PatternTuple,
    field_path: &[String],
    _node_ident: &Ident,
) -> TokenStream {
    let elements = &pattern.elements;
    // Use helper to process elements with collection strategy
    let (match_patterns, element_assertions) =
        process_tuple_elements(elements, "__tuple_elem_", true, field_path);

    quote! {
        #[allow(unreachable_patterns)]
        match #value_expr {
            (#(#match_patterns),*) => {
                #(#element_assertions)*
            },
            _ => unreachable!("Plain tuple match should always succeed"),
        }
    }
}

// Check if a path refers to Option::Some

/// Generate comparison assertion with error collection
fn generate_comparison_assertion_with_collection(
    value_expr: &TokenStream,
    comparison_pattern: &PatternComparison,
    path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    let op = &comparison_pattern.op;
    let expected = &comparison_pattern.expr;

    let is_index_operation = true;

    let span = expected.span();
    let comparison = if is_index_operation {
        // For index operations, avoid references on both sides
        match comparison_pattern.op {
            ComparisonOp::Less => quote_spanned! {span=> (#value_expr).lt(&(#expected)) },
            ComparisonOp::LessEqual => quote_spanned! {span=> (#value_expr).le(&(#expected)) },
            ComparisonOp::Greater => quote_spanned! {span=> (#value_expr).gt(&(#expected)) },
            ComparisonOp::GreaterEqual => quote_spanned! {span=> (#value_expr).ge(&(#expected)) },
            ComparisonOp::Equal => quote_spanned! {span=> (#value_expr).eq(&(#expected)) },
            ComparisonOp::NotEqual => quote_spanned! {span=> (#value_expr).ne(&(#expected)) },
        }
    } else {
        todo!()
    };

    let error_type_path = if matches!(op, ComparisonOp::Equal) {
        quote!(::assert_struct::__macro_support::ErrorType::Equality)
    } else {
        quote!(::assert_struct::__macro_support::ErrorType::Comparison)
    };

    let expected_value = if matches!(op, ComparisonOp::Equal) {
        let expected_str = quote! { #expected }.to_string();
        quote!(Some(#expected_str.to_string()))
    } else {
        quote!(None)
    };

    let error_push = generate_error_push(
        span,
        path,
        &comparison_pattern.to_error_context_string(),
        quote!(format!("{:?}", #value_expr)),
        error_type_path,
        expected_value,
        node_ident,
    );

    quote_spanned! {span=>
        #[allow(clippy::nonminimal_bool)]
        if !(#comparison) {
            #error_push
        }
    }
}

/// Generate assertion for enum tuple variants with error collection
fn generate_enum_tuple_assertion_with_collection(
    value_expr: &TokenStream,
    pattern: &PatternEnum,
    field_path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    let variant_path = &pattern.path;
    let elements = &pattern.elements;
    let field_path_str = field_path.join(".");
    let span = variant_path.span();

    // Special handling for unit variants (empty elements)
    if elements.is_empty() {
        quote_spanned! {span=>
            if !matches!(#value_expr, #variant_path) {
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
        base_path.push(variant_name);
        let use_variant_name = elements.len() == 1;

        // Use helper to process elements with appropriate path
        let (match_patterns, element_assertions) = if use_variant_name {
            // base_path.push(variant_name);
            process_tuple_elements(elements, "__elem_", true, &base_path)
        } else {
            process_tuple_elements(elements, "__elem_", true, &base_path)
        };

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
    }
}

/// Generate range assertion with error collection
fn generate_range_assertion_with_collection(
    value_expr: &TokenStream,
    range_pattern: &PatternRange,
    path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    let range = &range_pattern.expr;

    let span = range.span();
    let error_push = generate_error_push(
        span,
        path,
        &range_pattern.to_error_context_string(),
        quote!(format!("{:?}", #value_expr)),
        quote!(::assert_struct::__macro_support::ErrorType::Range),
        quote!(None),
        node_ident,
    );

    quote_spanned! {span=>
        match &#value_expr {
            #range => {},
            _ => {
                #error_push
            }
        }
    }
}

/// Generate the error context creation and push code
fn generate_error_push(
    span: proc_macro2::Span,
    field_path: &[String],
    pattern_str: &str,
    actual_value: TokenStream,
    error_type_path: TokenStream,
    expected_value: TokenStream,
    node_ident: &Ident,
) -> TokenStream {
    let field_path_str = field_path.join(".");
    quote_spanned! {span=>
        let __line = line!();
        let __file = file!();
        let __error = ::assert_struct::__macro_support::ErrorContext {
            field_path: #field_path_str.to_string(),
            pattern_str: #pattern_str.to_string(),
            actual_value: #actual_value,
            line_number: __line,
            file_name: __file,
            error_type: #error_type_path,
            expected_value: #expected_value,
            error_node: Some(&#node_ident),
        };
        __errors.push(__error);
    }
}

/// Generate string literal assertion with error collection
/// String literals always use .as_ref() to handle String/&str matching
fn generate_string_assertion_with_collection(
    value_expr: &TokenStream,
    string_pattern: &PatternString,
    path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    let lit = &string_pattern.lit;

    // String patterns always use .as_ref() to handle String/&str matching
    let span = lit.span();
    let pattern_str = string_pattern.to_error_context_string();
    let error_push = generate_error_push(
        span,
        path,
        &pattern_str,
        quote!(format!("{:?}", actual)),
        quote!(::assert_struct::__macro_support::ErrorType::Value),
        quote!(None),
        node_ident,
    );

    quote_spanned! {span=>
        let actual = (#value_expr).as_ref();
        if !matches!(actual, #lit) {
            #error_push
        }
    }
}

/// Generate simple assertion with error collection
fn generate_simple_assertion_with_collection(
    actual: &TokenStream,
    simple_pattern: &PatternSimple,
    path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    let expected = &simple_pattern.expr;
    let span = expected.span();
    let error_push = generate_error_push(
        span,
        path,
        &simple_pattern.to_error_context_string(),
        quote!(format!("{:?}", #actual)),
        quote!(::assert_struct::__macro_support::ErrorType::Value),
        quote!(None),
        node_ident,
    );

    quote_spanned! {span=>
        if !matches!(#actual, #expected) {
            #error_push
        }
    }
}

/// Generate slice assertion with error collection
fn generate_slice_assertion_with_collection(
    value_expr: &TokenStream,
    pattern: &PatternSlice,
    field_path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    let mut pattern_parts = Vec::new();
    let mut bindings_and_assertions = Vec::new();

    for (i, elem) in pattern.elements.iter().enumerate() {
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
    let elements_len = pattern.elements.len();

    let error_push = generate_error_push(
        proc_macro2::Span::call_site(),
        field_path,
        &format!("[{} elements]", elements_len),
        quote!(format!("{:?}", &#value_expr)),
        quote!(::assert_struct::__macro_support::ErrorType::Slice),
        quote!(None),
        node_ident,
    );

    quote! {
        match #slice_expr {
            [#(#pattern_parts),*] => {
                #(#bindings_and_assertions)*
            }
            _ => {
                #error_push
            }
        }
    }
}

#[cfg(feature = "regex")]
/// Generate regex assertion with error collection
fn generate_regex_assertion_with_collection(
    value_expr: &TokenStream,
    regex_pattern: &PatternRegex,
    path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    let pattern_str = &regex_pattern.pattern;
    let span = regex_pattern.span;

    let error_push = generate_error_push(
        span,
        path,
        &regex_pattern.to_error_context_string(),
        quote!(format!("{:?}", #value_expr)),
        quote!(::assert_struct::__macro_support::ErrorType::Regex),
        quote!(None),
        node_ident,
    );

    quote_spanned! {span=>
        {
            use ::assert_struct::Like;
            let re = ::assert_struct::__macro_support::Regex::new(#pattern_str)
                .expect(concat!("Invalid regex pattern: ", #pattern_str));
            if !#value_expr.like(&re) {
                #error_push
            }
        }
    }
}

#[cfg(feature = "regex")]
/// Generate Like trait assertion with error collection
fn generate_like_assertion_with_collection(
    value_expr: &TokenStream,
    like_pattern: &PatternLike,
    is_ref: bool,
    path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    let pattern_expr = &like_pattern.expr;
    let pattern_str = like_pattern.to_error_context_string();

    let span = pattern_expr.span();

    let actual_value = if is_ref {
        quote!(format!("{:?}", #value_expr))
    } else {
        quote!(format!("{:?}", &#value_expr))
    };

    let error_push = generate_error_push(
        span,
        path,
        &pattern_str,
        actual_value,
        quote!(::assert_struct::__macro_support::ErrorType::Regex),
        quote!(None),
        node_ident,
    );
    if is_ref {
        quote_spanned! {span=>
            {
                use ::assert_struct::Like;
                if !#value_expr.like(&#pattern_expr) {
                    #error_push
                }
            }
        }
    } else {
        quote_spanned! {span=>
            {
                use ::assert_struct::Like;
                if !(&#value_expr).like(&#pattern_expr) {
                    #error_push
                }
            }
        }
    }
}

/// Generate closure assertion with error collection
fn generate_closure_assertion_with_collection(
    value_expr: &TokenStream,
    closure_pattern: &PatternClosure,
    path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    let closure = &closure_pattern.closure;
    let span = closure.span();

    let error_push = generate_error_push(
        span,
        path,
        &quote! { #closure }.to_string(),
        quote!(format!("{:?}", #value_expr)),
        quote!(::assert_struct::__macro_support::ErrorType::Closure),
        quote!(None),
        node_ident,
    );

    quote_spanned! {span=>
        {
            if !::assert_struct::__macro_support::check_closure_condition(#value_expr, #closure) {
                #error_push
            }
        }
    }
}

/// Generate map assertion with error collection using duck typing
/// Assumes map types have len() -> usize and get(&K) -> Option<&V> methods
fn generate_map_assertion_with_collection(
    value_expr: &TokenStream,
    map_pattern: &PatternMap,
    _is_ref: bool,
    path: &[String],
    node_ident: &Ident,
) -> TokenStream {
    let entries = &map_pattern.entries;
    let rest = map_pattern.rest;

    // Use span from first entry or default span if empty
    let map_span = entries
        .first()
        .map(|(key, _)| key.span())
        .unwrap_or_else(proc_macro2::Span::call_site);

    // Generate length check assertion for exact matching (when no rest pattern)
    let len_check = if !rest {
        let expected_len = entries.len();
        let error_push = generate_error_push(
            map_span,
            path,
            &format!("#{{ {} entries }}", expected_len),
            quote!(format!("map with {} entries", (#value_expr).len())),
            quote!(::assert_struct::__macro_support::ErrorType::Value),
            quote!(Some(format!("{} entries", #expected_len))),
            node_ident,
        );
        quote_spanned! {map_span=>
            // Check exact length for maps without rest pattern
            if (#value_expr).len() != #expected_len {
                #error_push
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

            let missing_key_error = generate_error_push(
                span,
                &key_path,
                &format!("key: {}", key_str),
                quote!("missing key".to_string()),
                quote!(::assert_struct::__macro_support::ErrorType::Value),
                quote!(Some(format!("key present: {}", #key_str))),
                node_ident,
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
                        #missing_key_error
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
