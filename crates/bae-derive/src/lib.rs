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

extern crate proc_macro;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Attribute, ItemStruct, Result};

use bae_common::from_attributes_meta::{
    FromAttributesData, FromAttributesFieldData, FromAttributesMeta,
};

/// See root module docs for more info.
#[proc_macro_derive(FromAttributes, attributes(bae))]
pub fn from_attributes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    match from_attributes_impl(item) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn from_attributes_impl(item: ItemStruct) -> Result<TokenStream> {
    Ok(FromAttributesMeta::<Data, FieldData, false>::new(item)?.expand())
}

mod structs {
    use proc_macro2::Ident;

    use bae_derive_meta::FromAttributesInception;

    #[derive(FromAttributesInception, Debug, Default)]
    pub struct Bae {
        pub name: Option<Ident>,
    }
}

mod fields {
    use proc_macro2::Ident;

    use bae_derive_meta::FromAttributesInception;
    use syn::Path;

    #[derive(FromAttributesInception, Debug, Default)]
    pub struct Bae {
        pub name: Option<Ident>,
        pub skip: Option<()>,
        pub default: Option<Path>,
    }
}

struct Data(structs::Bae);

impl FromAttributesData for Data {
    fn new(attrs: &[Attribute]) -> Result<Self> {
        Ok(Data(
            structs::Bae::try_from_attributes(attrs)?.unwrap_or_default(),
        ))
    }
    fn rename_attr_name(&self, original: String) -> String {
        self.0
            .name
            .as_ref()
            .map(|name| name.to_string())
            .unwrap_or(original)
    }
}

struct FieldData(fields::Bae);

impl FromAttributesFieldData for FieldData {
    fn new(attrs: &[Attribute]) -> Result<Self> {
        Ok(FieldData(
            fields::Bae::try_from_attributes(attrs)?.unwrap_or_default(),
        ))
    }
    fn rename_field_name(&self, original: String) -> String {
        self.0
            .name
            .as_ref()
            .map(|name| name.to_string())
            .unwrap_or(original)
    }
    fn skip(&self) -> bool {
        self.0.skip.is_some()
    }
    fn default(&self) -> Option<TokenStream> {
        self.0.default.as_ref().map(|default| quote! { #default })
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
