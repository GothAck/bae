use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Result};

use bae_common::{
    from_attributes_meta::{
        FromAttributesFieldData, FromAttributesMeta, FromAttributesData,
    },
    FromAttributes,
};

pub type FromAttributesImpl = FromAttributesMeta<Data, FieldData, false>;

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

pub struct Data(structs::Bae);

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

pub struct FieldData(fields::Bae);

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
