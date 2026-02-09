//! Struct pattern types and utilities.
//!
//! Handles struct patterns like User { name: "Alice", age: 30, .. }

use std::fmt;
use syn::{Token, punctuated::Punctuated};

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
