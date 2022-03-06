use proc_macro2::TokenStream;
use syn::{parse_macro_input, Attribute, ItemStruct, Result};

use bae_common::from_attributes_meta::{
    FromAttributesData, FromAttributesFieldData, FromAttributesMeta,
};

type FromAttributesImpl = FromAttributesMeta<Data, FieldData, true>;

#[proc_macro_derive(FromAttributesInception, attributes(bae))]
pub fn from_attributes_inception(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    match FromAttributesImpl::new_and_expand(item) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

struct Data;

impl FromAttributesData for Data {
    fn new(_: &[Attribute]) -> Result<Self> {
        Ok(Data)
    }
    fn rename_attr_name(&self, original: String) -> String {
        original
    }
}

struct FieldData;

impl FromAttributesFieldData for FieldData {
    fn new(_: &[Attribute]) -> Result<Self> {
        Ok(FieldData)
    }
    fn rename_field_name(&self, original: String) -> String {
        original
    }
    fn skip(&self) -> bool {
        false
    }
    fn default(&self) -> Option<TokenStream> {
        None
    }
}
