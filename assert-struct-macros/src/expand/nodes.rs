//! Pattern node generation for error context and display.
//!
//! This module handles generating pattern node structures that are used for
//! displaying helpful error messages when assertions fail.

use crate::pattern::{
    ComparisonOp, Pattern, PatternClosure, PatternComparison, PatternEnum, PatternMap,
    PatternRange, PatternSimple, PatternSlice, PatternString, PatternStruct, PatternTuple,
    PatternWildcard, TupleElement,
};
#[cfg(feature = "regex")]
use crate::pattern::{PatternLike, PatternRegex};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

/// Get the node identifier for a pattern
pub(super) fn expand_pattern_node_ident(node_id: usize) -> Ident {
    Ident::new(&format!("__PATTERN_NODE_{}", node_id), Span::call_site())
}

/// Get the span for a pattern (if available)
pub(super) fn get_pattern_span(pattern: &Pattern) -> Option<Span> {
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
pub(super) fn generate_pattern_nodes(
    pattern: &Pattern,
    node_defs: &mut Vec<(usize, TokenStream)>,
    parent_ident: Option<&Ident>,
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
        let parent_ref = if let Some(parent) = parent_ident {
            quote! { Some(&#parent) }
        } else {
            quote! { None }
        };
        // Rest patterns don't have a specific span, use call_site
        let span = Span::call_site();
        return quote_spanned! { span=>
            ::assert_struct::__macro_support::PatternNode {
                kind: ::assert_struct::__macro_support::NodeKind::Rest,
                parent: #parent_ref,
                line: line!(),
                column: column!(),
            }
        };
    }

    let node_ident = Ident::new(&format!("__PATTERN_NODE_{}", node_id), Span::call_site());

    // Get the span for this pattern
    let span = get_pattern_span(pattern).unwrap_or_else(Span::call_site);

    // Generate parent reference
    let parent_ref = if let Some(parent) = parent_ident {
        quote! { Some(&#parent) }
    } else {
        quote! { None }
    };

    // Generate line and column using the pattern's span
    let line_col = quote_spanned! { span=>
        line: line!(),
        column: column!(),
    };

    let node_def = match pattern {
        Pattern::Simple(PatternSimple { expr, .. }) => {
            let value_str = quote! { #expr }.to_string();
            quote! {
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::Simple {
                        value: #value_str,
                    },
                    parent: #parent_ref,
                    #line_col
                }
            }
        }
        Pattern::String(PatternString { lit, .. }) => {
            let value_str = format!("\"{}\"", lit.value());
            quote! {
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::Simple {
                        value: #value_str,
                    },
                    parent: #parent_ref,
                    #line_col
                }
            }
        }
        Pattern::Comparison(PatternComparison { op, expr, .. }) => {
            let op_variant = match op {
                ComparisonOp::Less => quote!(::assert_struct::__macro_support::ComparisonOp::Less),
                ComparisonOp::LessEqual => quote!(::assert_struct::__macro_support::ComparisonOp::LessEqual),
                ComparisonOp::Greater => quote!(::assert_struct::__macro_support::ComparisonOp::Greater),
                ComparisonOp::GreaterEqual => quote!(::assert_struct::__macro_support::ComparisonOp::GreaterEqual),
                ComparisonOp::Equal => quote!(::assert_struct::__macro_support::ComparisonOp::Equal),
                ComparisonOp::NotEqual => quote!(::assert_struct::__macro_support::ComparisonOp::NotEqual),
            };
            let value_str = quote! { #expr }.to_string();
            quote! {
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::Comparison {
                        op: #op_variant,
                        value: #value_str,
                    },
                    parent: #parent_ref,
                    #line_col
                }
            }
        }
        Pattern::Range(PatternRange { expr, .. }) => {
            let pattern_str = quote! { #expr }.to_string();
            quote! {
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::Range {
                        pattern: #pattern_str,
                    },
                    parent: #parent_ref,
                    #line_col
                }
            }
        }
        #[cfg(feature = "regex")]
        Pattern::Regex(PatternRegex { pattern, .. }) => {
            let pattern_str = format!("r\"{}\"", pattern);
            quote! {
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::Regex {
                        pattern: #pattern_str,
                    },
                    parent: #parent_ref,
                    #line_col
                }
            }
        }
        #[cfg(feature = "regex")]
        Pattern::Like(PatternLike { expr, .. }) => {
            let expr_str = quote! { #expr }.to_string();
            quote! {
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::Like {
                        expr: #expr_str,
                    },
                    parent: #parent_ref,
                    #line_col
                }
            }
        }
        Pattern::Wildcard(PatternWildcard { .. }) => {
            quote! {
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::Wildcard,
                    parent: #parent_ref,
                    #line_col
                }
            }
        }
        Pattern::Closure(PatternClosure { closure, .. }) => {
            let closure_str = quote! { #closure }.to_string();
            quote! {
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::Closure {
                        closure: #closure_str,
                    },
                    parent: #parent_ref,
                    #line_col
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
                    generate_pattern_nodes(pattern, node_defs, Some(&node_ident))
                })
                .collect();

            let path_str = quote!(#path).to_string().replace(" :: ", "::");
            let args = if elements.is_empty() {
                quote!(None)
            } else {
                quote!(Some(&[#(&#child_refs),*]))
            };

            quote! {
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::EnumVariant {
                        path: #path_str,
                        args: #args,
                    },
                    parent: #parent_ref,
                    #line_col
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
                    generate_pattern_nodes(pattern, node_defs, Some(&node_ident))
                })
                .collect();

            quote! {
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::Tuple {
                        items: &[#(&#child_refs),*],
                    },
                    parent: #parent_ref,
                    #line_col
                }
            }
        }
        Pattern::Slice(PatternSlice { elements, .. }) => {
            let child_refs: Vec<TokenStream> = elements
                .iter()
                .map(|elem| generate_pattern_nodes(elem, node_defs, Some(&node_ident)))
                .collect();

            quote! {
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::Slice {
                        items: &[#(&#child_refs),*],
                    },
                    parent: #parent_ref,
                    #line_col
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
                    let child_ref = generate_pattern_nodes(&field.pattern, node_defs, Some(&node_ident));
                    quote! {
                        (#field_name, &#child_ref)
                    }
                })
                .collect();

            if *rest {
                // Rest patterns are handled inline, no need for a separate node
                let rest_line_col = quote_spanned! { span=>
                    line: line!(),
                    column: column!(),
                };
                quote! {
                    ::assert_struct::__macro_support::PatternNode {
                        kind: ::assert_struct::__macro_support::NodeKind::Struct {
                            name: #name_str,
                            fields: &[
                                #(#field_entries,)*
                                ("..", &::assert_struct::__macro_support::PatternNode {
                                    kind: ::assert_struct::__macro_support::NodeKind::Rest,
                                    parent: Some(&#node_ident),
                                    #rest_line_col
                                })
                            ],
                        },
                        parent: #parent_ref,
                        #line_col
                    }
                }
            } else {
                quote! {
                    ::assert_struct::__macro_support::PatternNode {
                        kind: ::assert_struct::__macro_support::NodeKind::Struct {
                            name: #name_str,
                            fields: &[#(#field_entries),*],
                        },
                        parent: #parent_ref,
                        #line_col
                    }
                }
            }
        }
        Pattern::Map(PatternMap { entries, rest, .. }) => {
            let entry_refs: Vec<TokenStream> = entries
                .iter()
                .map(|(key, value)| {
                    let key_str = quote! { #key }.to_string();
                    let value_ref = generate_pattern_nodes(value, node_defs, Some(&node_ident));
                    quote! {
                        (#key_str, &#value_ref)
                    }
                })
                .collect();

            if *rest {
                let rest_line_col = quote_spanned! { span=>
                    line: line!(),
                    column: column!(),
                };
                quote! {
                    ::assert_struct::__macro_support::PatternNode {
                        kind: ::assert_struct::__macro_support::NodeKind::Map {
                            entries: &[
                                #(#entry_refs,)*
                                ("..", &::assert_struct::__macro_support::PatternNode {
                                    kind: ::assert_struct::__macro_support::NodeKind::Rest,
                                    parent: Some(&#node_ident),
                                    #rest_line_col
                                })
                            ],
                        },
                        parent: #parent_ref,
                        #line_col
                    }
                }
            } else {
                quote! {
                    ::assert_struct::__macro_support::PatternNode {
                        kind: ::assert_struct::__macro_support::NodeKind::Map {
                            entries: &[#(#entry_refs),*],
                        },
                        parent: #parent_ref,
                        #line_col
                    }
                }
            }
        }
    };

    node_defs.push((node_id, node_def));
    quote! { #node_ident }
}
