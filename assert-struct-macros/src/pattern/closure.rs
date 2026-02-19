//! Closure pattern types.
//!
//! Handles closure patterns: |x| x > 5

use syn::parse::Parse;

use crate::parse::next_node_id;

/// Closure pattern: |x| expr for custom validation (escape hatch)
#[derive(Debug, Clone)]
pub(crate) struct PatternClosure {
    pub node_id: usize,
    pub closure: syn::ExprClosure,
}

impl Parse for PatternClosure {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Closure pattern: |x| expr or move |x| expr for custom validation (escape hatch)
        // Examples: `|x| x > 5`, `move |x| complex_logic(x)`, `|x| { x.len() > 0 }`
        let closure: syn::ExprClosure = input.parse()?;

        // Validate: exactly one parameter
        if closure.inputs.len() != 1 {
            return Err(syn::Error::new_spanned(
                &closure.inputs,
                "Closure must have exactly one parameter",
            ));
        }

        Ok(PatternClosure {
            node_id: next_node_id(),
            closure,
        })
    }
}
