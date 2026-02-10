//! Struct pattern types and utilities.
//!
//! Handles struct patterns like User { name: "Alice", age: 30, .. }

use std::fmt;
use syn::{Token, parse::Parse, punctuated::Punctuated};

use crate::parse::next_node_id;
use crate::pattern::{FieldAssertion, path_to_string};

/// Struct pattern: User { name: "Alice", age: 30, .. }
/// When path is None, it's a wildcard pattern: _ { name: "Alice", .. }
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
    /// _ { name: "Alice", .. }  // wildcard struct
    /// ```
    ///
    /// Handles both named structs (with a path) and wildcard structs (starting with `_`).
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let node_id = next_node_id();

        // Check if this is a wildcard struct pattern: _ { ... }
        let (path, wildcard_span) = if input.peek(Token![_]) {
            let underscore: Token![_] = input.parse()?;
            (None, Some(underscore.span)) // wildcard has no path
        } else {
            // Named struct pattern: TypeName { ... }
            (Some(input.parse::<syn::Path>()?), None)
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
                let _: Token![..] = content.parse()?;
                rest = true;
                break;
            }
        }

        // Wildcard struct patterns must use rest pattern (..)
        // to indicate partial matching
        if path.is_none() && !rest {
            return Err(syn::Error::new(
                wildcard_span.unwrap(),
                "Wildcard struct patterns must use '..' for partial matching",
            ));
        }

        Ok(PatternStruct {
            node_id,
            path,
            fields,
            rest,
        })
    }
}

impl fmt::Display for PatternStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "{} {{ ", path_to_string(path))?;
        } else {
            write!(f, "_ {{ ")?;
        }
        for (i, field) in self.fields.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}: {}", field.field_name, field.pattern)?;
        }
        if self.rest {
            if !self.fields.is_empty() {
                write!(f, ", ")?;
            }
            write!(f, "..")?;
        }
        write!(f, " }}")
    }
}
