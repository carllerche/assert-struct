//! Tuple pattern types and utilities.
//!
//! Handles tuple patterns including enum variants: Some(42), Event::Click(>= 0, < 100)

use std::fmt;

use crate::pattern::{FieldOperation, Pattern, path_to_string};

/// Tuple pattern: (10, 20) or Some(42) or None
/// Supports mixed positional and indexed elements
#[derive(Debug, Clone)]
pub(crate) struct PatternTuple {
    pub node_id: usize,
    pub path: Option<syn::Path>,
    pub elements: Vec<TupleElement>,
}

/// Represents an element in a tuple pattern, supporting both positional and indexed syntax
#[derive(Debug, Clone)]
pub(crate) enum TupleElement {
    /// Positional element: just a pattern in sequence
    /// Example: "foo", > 10, Some(42)
    Positional { pattern: Pattern },

    /// Indexed element: explicit index with optional operations
    /// Example: 0: "foo", *1: "bar", 2.len(): 5
    Indexed {
        index: usize,
        operations: Option<FieldOperation>,
        pattern: Pattern,
    },
}

impl fmt::Display for PatternTuple {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(path) = &self.path {
            write!(f, "{}", path_to_string(path))?;
        }
        write!(f, "(")?;
        for (i, elem) in self.elements.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", elem)?;
        }
        write!(f, ")")
    }
}

impl fmt::Display for TupleElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TupleElement::Positional { pattern } => {
                write!(f, "{}", pattern)
            }
            TupleElement::Indexed {
                index,
                operations,
                pattern,
            } => {
                if let Some(ops) = operations {
                    match ops {
                        FieldOperation::Deref { count, .. } => {
                            // Show deref operations before the index: *0:
                            for _ in 0..*count {
                                write!(f, "*")?;
                            }
                            write!(f, "{}: {}", index, pattern)
                        }
                        FieldOperation::Method { name, .. } => {
                            // Show method calls after the index: 0.len():
                            write!(f, "{}.{}(): {}", index, name, pattern)
                        }
                        FieldOperation::Await { .. } => {
                            // Show await after the index: 0.await:
                            write!(f, "{}.await: {}", index, pattern)
                        }
                        FieldOperation::Nested { fields, .. } => {
                            // Show nested access after the index: 0.field:
                            write!(f, "{}", index)?;
                            for field in fields {
                                write!(f, ".{}", field)?;
                            }
                            write!(f, ": {}", pattern)
                        }
                        FieldOperation::Index { index: idx, .. } => {
                            // Show index access after the tuple index: 0[1]:
                            write!(f, "{}[{}]: {}", index, quote::quote! { #idx }, pattern)
                        }
                        FieldOperation::Chained { operations, .. } => {
                            // Show chained operations after the tuple index: 0.field[1]:
                            write!(f, "{}", index)?;
                            for op in operations {
                                match op {
                                    FieldOperation::Nested { fields, .. } => {
                                        for field in fields {
                                            write!(f, ".{}", field)?;
                                        }
                                    }
                                    FieldOperation::Method { name, .. } => {
                                        write!(f, ".{}()", name)?;
                                    }
                                    FieldOperation::Await { .. } => {
                                        write!(f, ".await")?;
                                    }
                                    FieldOperation::Index { index, .. } => {
                                        write!(f, "[{}]", quote::quote! { #index })?;
                                    }
                                    _ => write!(f, "{}", op)?,
                                }
                            }
                            write!(f, ": {}", pattern)
                        }
                        FieldOperation::Combined {
                            deref_count,
                            operation,
                            ..
                        } => {
                            // Show combined operations: *0.len():
                            for _ in 0..*deref_count {
                                write!(f, "*")?;
                            }
                            match operation.as_ref() {
                                FieldOperation::Method { name, .. } => {
                                    write!(f, "{}.{}(): {}", index, name, pattern)
                                }
                                _ => {
                                    write!(f, "{}{}: {}", index, operation, pattern)
                                }
                            }
                        }
                    }
                } else {
                    write!(f, "{}: {}", index, pattern)
                }
            }
        }
    }
}
