//! Struct pattern types and utilities.
//!
//! Handles struct patterns like User { name: "Alice", age: 30, .. }

use syn::{Token, parse::Parse, punctuated::Punctuated};

use crate::parse::next_node_id;
use crate::pattern::FieldAssertion;

/// Struct pattern: User { name: "Alice", age: 30, .. }
/// When path is None, it's an anonymous struct pattern: { name: "Alice" }
#[derive(Debug, Clone)]
pub(crate) struct PatternStruct {
    pub node_id: usize,
    pub path: Option<syn::Path>,
    pub fields: Punctuated<FieldAssertion, Token![,]>,
    pub rest: bool,
}

impl Parse for PatternStruct {
    /// Parses a struct pattern with braces.
    ///
    /// # Example Input
    /// ```text
    /// User { name: "Alice", age: >= 18, .. }
    /// { name: "Alice" }        // anonymous struct (always partial)
    /// ```
    ///
    /// Handles both named and anonymous structs.
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let node_id = next_node_id();

        let path = if input.peek(syn::token::Brace) {
            None // anonymous struct
        } else {
            // Named struct pattern: TypeName { ... }
            Some(input.parse::<syn::Path>()?)
        };

        // Parse the braced contents
        let content;
        syn::braced!(content in input);

        // Parse comma-separated field assertions with optional rest pattern (..)
        let mut fields = Punctuated::new();
        let mut rest = false;

        while !content.is_empty() {
            // Check for rest pattern (..) which allows partial matching
            if content.peek(Token![..]) {
                if path.is_none() {
                    return Err(content.error("unexpected token in anonymous struct pattern"));
                }
                let _: Token![..] = content.parse()?;
                rest = true;
                break;
            }

            fields.push_value(content.parse()?);

            if content.is_empty() {
                break;
            }

            let comma: Token![,] = content.parse()?;
            fields.push_punct(comma);

            // Rest pattern can appear after a comma
            if content.peek(Token![..]) {
                if path.is_none() {
                    return Err(content.error("unexpected token in anonymous struct pattern"));
                }
                let _: Token![..] = content.parse()?;
                rest = true;
                break;
            }
        }

        // Anonymous struct patterns always use partial matching because their type is unknown.
        if path.is_none() {
            rest = true;
        }

        Ok(PatternStruct {
            node_id,
            path,
            fields,
            rest,
        })
    }
}
