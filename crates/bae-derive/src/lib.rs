//! Derive macros for [`bae`](https://crates.io/crates/bae).

#![allow(clippy::let_and_return)]
#![deny(
    unused_variables,
    mutable_borrow_reservation_conflict,
    dead_code,
    unused_must_use,
    unused_imports,
    missing_docs
)]

mod from_attributes;

extern crate proc_macro;

use syn::{parse_macro_input, ItemStruct};

use crate::from_attributes::FromAttributesImpl;

/// See root module docs for more info.
#[proc_macro_derive(FromAttributes, attributes(bae))]
pub fn from_attributes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    match FromAttributesImpl::new_and_expand(item) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

#[cfg(test)]
mod test {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_ui() {
        let t = trybuild::TestCases::new();
        t.pass("tests/compile_pass/*.rs");
        t.compile_fail("tests/compile_fail/*.rs");
    }
}
