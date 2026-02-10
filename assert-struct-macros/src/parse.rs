use crate::AssertStruct;
use std::cell::Cell;
use syn::{Result, Token, parse::Parse, parse::ParseStream};

thread_local! {
    static NODE_ID_COUNTER: Cell<usize> = const { Cell::new(0) };
}

pub(crate) fn next_node_id() -> usize {
    NODE_ID_COUNTER.with(|counter| {
        let id = counter.get();
        counter.set(id + 1);
        id
    })
}

fn reset_node_counter() {
    NODE_ID_COUNTER.with(|counter| counter.set(0));
}

impl Parse for AssertStruct {
    /// Parses the top-level macro invocation.
    ///
    /// # Example Input
    /// ```text
    /// assert_struct!(value, Pattern { field: matcher, .. })
    /// assert_struct!(value, Some(> 30))
    /// assert_struct!(value, [1, 2, 3])
    /// ```
    ///
    /// The macro always expects: `expression`, `pattern`
    fn parse(input: ParseStream) -> Result<Self> {
        // Reset the node ID counter for each macro invocation
        reset_node_counter();

        let value = input.parse()?;
        let _: Token![,] = input.parse()?;
        let pattern = input.parse()?;

        Ok(AssertStruct { value, pattern })
    }
}
