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
use quote::quote;
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
            ::assert_struct::__macro_support::PatternNode {
                kind: ::assert_struct::__macro_support::NodeKind::Rest,
            }
        };
    }

    let node_ident = Ident::new(&format!("__PATTERN_NODE_{}", node_id), Span::call_site());

    let node_def = match pattern {
        Pattern::Simple(PatternSimple { expr, .. }) => {
            let value_str = quote! { #expr }.to_string();
            quote! {
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::Simple {
                        value: #value_str,
                    }
                }
            }
        }
        Pattern::String(PatternString { lit, .. }) => {
            let value_str = format!("\"{}\"", lit.value());
            quote! {
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::Simple {
                        value: #value_str,
                    }
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
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::Comparison {
                        op: #op_str,
                        value: #value_str,
                    }
                }
            }
        }
        Pattern::Range(PatternRange { expr, .. }) => {
            let pattern_str = quote! { #expr }.to_string();
            quote! {
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::Range {
                        pattern: #pattern_str,
                    }
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
                    }
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
                    }
                }
            }
        }
        Pattern::Wildcard(PatternWildcard { .. }) => {
            quote! {
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::Wildcard
                }
            }
        }
        Pattern::Closure(PatternClosure { closure, .. }) => {
            let closure_str = quote! { #closure }.to_string();
            quote! {
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::Closure {
                        closure: #closure_str,
                    }
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
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::EnumVariant {
                        path: #path_str,
                        args: #args,
                    }
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
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::Tuple {
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

            quote! {
                ::assert_struct::__macro_support::PatternNode {
                    kind: ::assert_struct::__macro_support::NodeKind::Slice {
                        items: &[#(&#child_refs),*],
                    }
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
                    ::assert_struct::__macro_support::PatternNode {
                        kind: ::assert_struct::__macro_support::NodeKind::Struct {
                            name: #name_str,
                            fields: &[
                                #(#field_entries,)*
                                ("..", &::assert_struct::__macro_support::PatternNode {
                                    kind: ::assert_struct::__macro_support::NodeKind::Rest
                                })
                            ],
                        }
                    }
                }
            } else {
                quote! {
                    ::assert_struct::__macro_support::PatternNode {
                        kind: ::assert_struct::__macro_support::NodeKind::Struct {
                            name: #name_str,
                            fields: &[#(#field_entries),*],
                        }
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
                    ::assert_struct::__macro_support::PatternNode {
                        kind: ::assert_struct::__macro_support::NodeKind::Map {
                            entries: &[
                                #(#entry_refs,)*
                                ("..", &::assert_struct::__macro_support::PatternNode {
                                    kind: ::assert_struct::__macro_support::NodeKind::Rest
                                })
                            ],
                        }
                    }
                }
            } else {
                quote! {
                    ::assert_struct::__macro_support::PatternNode {
                        kind: ::assert_struct::__macro_support::NodeKind::Map {
                            entries: &[#(#entry_refs),*],
                        }
                    }
                }
            }
        }
    };

    node_defs.push((node_id, node_def));
    quote! { #node_ident }
}
