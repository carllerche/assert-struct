mod nodes;

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

use nodes::{expand_pattern_node_ident, generate_pattern_nodes};

pub fn expand(assert: &AssertStruct) -> TokenStream {
    let value = &assert.value;
    let pattern = &assert.pattern;

    // Generate pattern nodes using the node IDs from the patterns
    let mut node_defs = Vec::new();
    let root_ref = generate_pattern_nodes(pattern, &mut node_defs, None);

    // Generate static declarations for all nodes
    let node_constants: Vec<TokenStream> = node_defs
        .iter()
        .map(|(id, def)| {
            let ident = Ident::new(&format!("__PATTERN_NODE_{}", id), Span::call_site());
            quote! {
                static #ident: ::assert_struct::__macro_support::PatternNode = #def;
            }
        })
        .collect();

    let assertion = expand_pattern_assertion(&quote! { #value }, pattern);

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

                // Create error report. Both values are compile-time constants:
                // - CARGO_MANIFEST_DIR: absolute path to this package's root
                // - file!(): path relative to the workspace root
                // Together they let us derive the absolute source path at runtime
                // without relying on the working directory.
                let mut __report = ::assert_struct::__macro_support::ErrorReport::new(
                    ::std::env!("CARGO_MANIFEST_DIR"),
                    ::std::file!(),
                );

                #assertion

                // Check if any errors were collected
                if !__report.is_empty() {
                    panic!("{}", __report);
                }
            };
            __assert_struct_result
        }
    }
}

/// Generate assertion code with error collection instead of immediate panic.
fn expand_pattern_assertion(value_expr: &TokenStream, pattern: &Pattern) -> TokenStream {
    match pattern {
        Pattern::Simple(simple_pattern) => expand_simple_assertion(value_expr, simple_pattern),
        Pattern::String(string_pattern) => expand_string_assertion(value_expr, string_pattern),
        Pattern::Struct(struct_pattern) => expand_struct_assertion(value_expr, struct_pattern),
        Pattern::Comparison(comparison_pattern) => {
            expand_comparison_assertion(value_expr, comparison_pattern)
        }
        Pattern::Enum(enum_pattern) => {
            // Enum tuple variant - use collection version
            expand_enum_assertion(value_expr, enum_pattern)
        }
        Pattern::Tuple(tuple_pattern) => {
            // Plain tuple - use collection version for proper error collection
            expand_tuple_assertion(value_expr, tuple_pattern)
        }
        Pattern::Wildcard(_) => {
            // Wildcard patterns generate no assertions - they just verify the field exists
            // which is already handled by the struct/tuple destructuring
            quote! {}
        }
        Pattern::Range(range_pattern) => {
            // Generate improved range assertion with error collection
            expand_range_assertion(value_expr, range_pattern)
        }
        Pattern::Slice(slice_pattern) => {
            // Generate slice assertion with error collection
            expand_slice_assertion(value_expr, slice_pattern)
        }
        #[cfg(feature = "regex")]
        Pattern::Regex(regex_pattern) => {
            // Generate regex assertion with error collection
            expand_regex_assertion(value_expr, regex_pattern)
        }
        #[cfg(feature = "regex")]
        Pattern::Like(like_pattern) => {
            // Generate Like trait assertion with error collection
            expand_like_assertion(value_expr, like_pattern)
        }
        Pattern::Closure(closure_pattern) => {
            // Generate closure assertion with error collection
            expand_closure_assertion(value_expr, closure_pattern)
        }
        Pattern::Map(map_pattern) => {
            // Generate map assertion with error collection
            expand_map_assertion(value_expr, map_pattern)
        }
    }
}

/// Generate struct assertion with error collection for multiple field failures
fn expand_struct_assertion(value_expr: &TokenStream, pattern: &PatternStruct) -> TokenStream {
    let struct_path = &pattern.path;
    let fields = &pattern.fields;
    let rest = pattern.rest;

    // If struct_path is None, it's a wildcard pattern - use field access
    let Some(struct_path) = struct_path.as_ref() else {
        return expand_struct_wildcard_assertion(value_expr, fields);
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
            let assertion = expand_field_assertion(&quote! { #field_name }, f);

            // Wrap the assertion with the span of the field pattern if available
            if let Some(span) = f.pattern.span() {
                quote_spanned! {span=> #assertion }
            } else {
                assertion
            }
        })
        .collect();

    let span = struct_path.span();

    let error_push = generate_error_push(
        span,
        quote!(format!("{:?}", #value_expr)),
        quote!(None),
        pattern.node_id,
    );

    quote_spanned! {span=>
        #[allow(unreachable_patterns)]
        match &#value_expr {
            #struct_path { #(#field_names),* #rest_pattern } => {
                #(#field_assertions)*
            },
            _ => {
                #error_push
            }
        }
    }
}

/// Generate wildcard struct assertion using direct field access
fn expand_struct_wildcard_assertion(
    value_expr: &TokenStream,
    fields: &Punctuated<FieldAssertion, Token![,]>,
) -> TokenStream {
    let field_assertions: Vec<_> = fields
        .iter()
        .map(|f| {
            let field_name = f.operations.root_field_name();
            let field_pattern = &f.pattern;
            let field_operations = &f.operations;

            // Access the field and apply tail operations
            let base_field_access = quote! { (#value_expr).#field_name };

            let expr = if let Some(tail_ops) = field_operations.tail_operations() {
                // Apply remaining operations after the field access
                apply_field_operations(&base_field_access, &tail_ops)
            } else {
                // No additional operations, take a reference to the field for comparison
                quote! { &#base_field_access }
            };

            // Recursively expand the pattern for this field
            expand_pattern_assertion(&expr, field_pattern)
        })
        .collect();

    quote! {
        #(#field_assertions)*
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
fn expand_field_assertion(base: &TokenStream, field_assertion: &FieldAssertion) -> TokenStream {
    let field_operations = &field_assertion.operations;
    let field_pattern = &field_assertion.pattern;

    // Apply tail operations and determine final reference context
    let expr = if let Some(tail_ops) = field_operations.tail_operations() {
        apply_field_operations(base, &tail_ops)
    } else {
        base.clone()
    };

    // Generate the pattern assertion
    expand_pattern_assertion(&expr, field_pattern)
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

                        // Generate assertion with error collection
                        let assertion = expand_pattern_assertion(&quote! { #name }, pattern);
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
                        let assertion = expand_field_assertion(&quote! { #name }, boxed_elem);
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
fn expand_tuple_assertion(value_expr: &TokenStream, pattern: &PatternTuple) -> TokenStream {
    let elements = &pattern.elements;
    // Use helper to process elements with collection strategy
    let (match_patterns, element_assertions) = process_tuple_elements(elements, "__tuple_elem_");

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
fn expand_comparison_assertion(
    value_expr: &TokenStream,
    pattern: &PatternComparison,
) -> TokenStream {
    let op = &pattern.op;
    let expected = &pattern.expr;

    let span = expected.span();

    let comparison = {
        // For index operations, avoid references on both sides
        match &pattern.op {
            ComparisonOp::Less(_) => quote_spanned! {span=> (#value_expr).lt(&(#expected)) },
            ComparisonOp::LessEqual(_) => quote_spanned! {span=> (#value_expr).le(&(#expected)) },
            ComparisonOp::Greater(_) => quote_spanned! {span=> (#value_expr).gt(&(#expected)) },
            ComparisonOp::GreaterEqual(_) => quote_spanned! {span=> (#value_expr).ge(&(#expected)) },
            ComparisonOp::Equal(_) => quote_spanned! {span=> (#value_expr).eq(&(#expected)) },
            ComparisonOp::NotEqual(_) => quote_spanned! {span=> (#value_expr).ne(&(#expected)) },
        }
    };

    let expected_value = if matches!(op, ComparisonOp::Equal(_)) {
        let expected_str = quote! { #expected }.to_string();
        quote!(Some(#expected_str.to_string()))
    } else {
        quote!(None)
    };

    let error_push = generate_error_push(
        span,
        quote!(format!("{:?}", #value_expr)),
        expected_value,
        pattern.node_id,
    );

    quote_spanned! {span=>
        #[allow(clippy::nonminimal_bool)]
        if !(#comparison) {
            #error_push
        }
    }
}

/// Generate assertion for enum tuple variants with error collection
fn expand_enum_assertion(value_expr: &TokenStream, pattern: &PatternEnum) -> TokenStream {
    let variant_path = &pattern.path;
    let elements = &pattern.elements;
    let span = variant_path.span();

    let error_push = generate_error_push(
        span,
        quote!(format!("{:?}", #value_expr)),
        quote!(None),
        pattern.node_id,
    );

    // Special handling for unit variants (empty elements)
    if elements.is_empty() {
        quote_spanned! {span=>
            if !matches!(#value_expr, #variant_path) {
                #error_push
            }
        }
    } else {
        // Use helper to process elements with appropriate path
        let (match_patterns, element_assertions) = process_tuple_elements(elements, "__elem_");

        quote_spanned! {span=>
            match &#value_expr {
                #variant_path(#(#match_patterns),*) => {
                    #(#element_assertions)*
                },
                _ => {
                    #error_push
                }
            }
        }
    }
}

/// Generate range assertion with error collection
fn expand_range_assertion(value_expr: &TokenStream, pattern: &PatternRange) -> TokenStream {
    let range = &pattern.expr;

    let span = range.span();
    let error_push = generate_error_push(
        span,
        quote!(format!("{:?}", #value_expr)),
        quote!(None),
        pattern.node_id,
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

/// Generate string literal assertion with error collection
/// String literals always use .as_ref() to handle String/&str matching
fn expand_string_assertion(value_expr: &TokenStream, pattern: &PatternString) -> TokenStream {
    let lit = &pattern.lit;

    // String patterns always use .as_ref() to handle String/&str matching
    let span = lit.span();
    let error_push = generate_error_push(
        span,
        quote!(format!("{:?}", actual)),
        quote!(None),
        pattern.node_id,
    );

    quote_spanned! {span=> {
        let actual = (#value_expr).as_ref();
        if !matches!(actual, #lit) {
            #error_push
        }
    }}
}

/// Generate simple assertion with error collection
fn expand_simple_assertion(actual: &TokenStream, pattern: &PatternSimple) -> TokenStream {
    let expected = &pattern.expr;
    let span = expected.span();
    let error_push = generate_error_push(
        span,
        quote!(format!("{:?}", #actual)),
        quote!(None),
        pattern.node_id,
    );

    quote_spanned! {span=>
        if !matches!(#actual, #expected) {
            #error_push
        }
    }
}

/// Generate slice assertion with error collection
fn expand_slice_assertion(value_expr: &TokenStream, pattern: &PatternSlice) -> TokenStream {
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

                let assertion = expand_pattern_assertion(&quote! { #binding }, elem);
                bindings_and_assertions.push(assertion);
            }
        }
    }

    // Convert Vec to slice for matching
    let slice_expr = quote! { (#value_expr).as_slice() };

    let error_push = generate_error_push(
        proc_macro2::Span::call_site(),
        quote!(format!("{:?}", &#value_expr)),
        quote!(None),
        pattern.node_id,
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
fn expand_regex_assertion(value_expr: &TokenStream, pattern: &PatternRegex) -> TokenStream {
    let pattern_str = &pattern.pattern;
    let span = pattern.span;

    let error_push = generate_error_push(
        span,
        quote!(format!("{:?}", #value_expr)),
        quote!(None),
        pattern.node_id,
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
fn expand_like_assertion(value_expr: &TokenStream, pattern: &PatternLike) -> TokenStream {
    let pattern_expr = &pattern.expr;

    let span = pattern_expr.span();
    let actual_value = quote!(format!("{:?}", #value_expr));

    let error_push = generate_error_push(span, actual_value, quote!(None), pattern.node_id);

    quote_spanned! {span=>
        {
            use ::assert_struct::Like;
            if !#value_expr.like(&#pattern_expr) {
                #error_push
            }
        }
    }
}

/// Generate closure assertion with error collection
fn expand_closure_assertion(value_expr: &TokenStream, pattern: &PatternClosure) -> TokenStream {
    let closure = &pattern.closure;
    let span = closure.span();

    let error_push = generate_error_push(
        span,
        quote!(format!("{:?}", #value_expr)),
        quote!(None),
        pattern.node_id,
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
fn expand_map_assertion(value_expr: &TokenStream, pattern: &PatternMap) -> TokenStream {
    let entries = &pattern.entries;
    let rest = pattern.rest;

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
            quote!(format!("map with {} entries", (#value_expr).len())),
            quote!(Some(format!("{} entries", #expected_len))),
            pattern.node_id,
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

            let span = key.span();
            let pattern_assertion =
                expand_pattern_assertion(&quote! { __map_value }, value_pattern);

            let missing_key_error = generate_error_push(
                span,
                quote!("missing key".to_string()),
                quote!(Some(format!("key present: {}", #key_str))),
                pattern.node_id,
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

/// Generate the error context creation and push code
fn generate_error_push(
    span: proc_macro2::Span,
    actual_value: TokenStream,
    expected_value: TokenStream,
    node_id: usize,
) -> TokenStream {
    let node_ident = expand_pattern_node_ident(node_id);
    quote_spanned! {span=>
        __report.push(&#node_ident, line!(), #actual_value, #expected_value);
    }
}
