use proc_macro::TokenStream;
use syn::{Expr, Token, punctuated::Punctuated};

mod expand;
mod parse;

// Root-level structs that track input details
struct AssertStruct {
    value: Expr,
    type_name: syn::Path,
    expected: Expected,
}

struct Expected {
    fields: Punctuated<FieldAssertion, Token![,]>,
    rest: bool, // true if ".." was present
}

enum FieldAssertion {
    Simple {
        field_name: syn::Ident,
        expected_value: Expr,
    },
    Nested {
        field_name: syn::Ident,
        type_name: syn::Path,
        nested: Expected,
    },
    Tuple {
        field_name: syn::Ident,
        elements: Vec<Expr>,
    },
    #[cfg(feature = "regex")]
    Regex {
        field_name: syn::Ident,
        pattern: String,
    },
    Comparison {
        field_name: syn::Ident,
        op: ComparisonOp,
        value: Expr,
    },
}

#[derive(Clone, Copy)]
enum ComparisonOp {
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
}

/// Assert that a struct matches the expected pattern.
///
/// # Examples
///
/// ```
/// use assert_struct::assert_struct;
///
/// #[derive(Clone)]
/// struct User {
///     name: String,
///     age: u32,
/// }
///
/// let user = User {
///     name: "Alice".to_string(),
///     age: 30,
/// };
///
/// // Exhaustive check - all fields must be specified
/// assert_struct!(user.clone(), User {
///     name: "Alice",
///     age: 30,
/// });
///
/// // Partial check - only specified fields are checked
/// assert_struct!(user, User {
///     name: "Alice",
///     ..
/// });
/// ```
#[proc_macro]
pub fn assert_struct(input: TokenStream) -> TokenStream {
    // Parse the input
    let assert = match parse::parse(input) {
        Ok(assert) => assert,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };

    // Expand to output code
    let expanded = expand::expand(&assert);

    TokenStream::from(expanded)
}
