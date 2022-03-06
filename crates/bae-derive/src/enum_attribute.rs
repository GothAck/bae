use proc_macro2::TokenStream;
use quote::quote;
use syn::{ItemEnum, Result, Variant, Fields, Error, spanned::Spanned};

use bae_common::{private::IterCombineSynErrors, FromAttributes};

pub struct EnumAttribute {
    item: ItemEnum,
    #[allow(dead_code)]
    attr: enums::Bae,
    variants: Vec<EnumAttributeVariant>,
}

impl EnumAttribute {
    pub fn new_and_expand(item: ItemEnum) -> Result<TokenStream> {
        Ok(Self::new(item)?
            .expand())
    }

    pub fn new(item: ItemEnum) -> Result<Self> {
        let attr = enums::Bae::try_from_attributes(&item.attrs)?
            .unwrap_or_default();

        let variants = item.variants
            .iter()
            .map(EnumAttributeVariant::new)
            .collect_syn_error()?;

        Ok(Self {
            item,
            attr,
            variants,
        })
    }

    pub fn expand(&self) -> TokenStream {
        let ident = &self.item.ident;

        let match_arms: TokenStream = self.variants
            .iter()
            .map(EnumAttributeVariant::expand_match_arms)
            .collect();

        let supported_values = {
            let mut values = self
                .variants
                .iter()
                .filter_map(EnumAttributeVariant::to_supported_value)
                .collect::<Vec<_>>();
            values.sort_unstable();
            values.join(", ")
        };

        quote! {
            impl ::bae::BaeParse for #ident {
                fn parse(input: ::bae::private::syn::parse::ParseStream, ctx: &::bae::BaeParseCtx) -> ::bae::BaeParseResult<Self> {
                    let variant_name: ::bae::private::syn::Ident = input.parse()?;

                    let variant = match &*variant_name.to_string() {
                        #match_arms
                        _ => {
                            return Err(::bae::private::syn::Error::new(
                                variant_name.span(),
                                format!(
                                    "Invalid value, supported values are {}",
                                    #supported_values,
                                ),
                            ));
                        }
                    };

                    Ok(::bae::BaeSpanned::new(variant, variant_name.span()))
                }
            }

            impl ::bae::BaeDefault for #ident {
                fn bae_default() -> ::bae::BaeDefaultedValue<Self> {
                    ::bae::BaeDefaultedValue::NoDefault
                }
            }
        }
    }
}

mod enums {
    use bae_derive_meta::FromAttributesInception;

    #[derive(FromAttributesInception, Debug, Default)]
    pub struct Bae {}
}

struct EnumAttributeVariant {
    item: Variant,
    attr: variants::Bae,
}

impl EnumAttributeVariant {
    fn new(item: &Variant) -> Result<Self> {
        match item.fields {
            Fields::Unit => {},
            Fields::Named(..)
            | Fields::Unnamed(..) => return Err(Error::new(
                item.span(),
                "Only Unit variants are supported",
            ))
        }

        let attr = variants::Bae::try_from_attributes(&item.attrs)?
            .unwrap_or_default();

        Ok(Self {
            item: item.clone(),
            attr,
        })
    }

    fn expand_match_arms(&self) -> TokenStream {
        let ident = &self.item.ident;
        let pattern = ident.to_string();

        if self.attr.skip.is_some() {
            quote! {}
        } else {
            quote! {
                #pattern => Self::#ident,
            }
        }
    }

    fn to_supported_value(&self) -> Option<String> {
        if self.attr.skip.is_some() {
            None
        } else {
            Some(format!("`{}`", self.item.ident))
        }
    }
}

mod variants {
    use bae_derive_meta::FromAttributesInception;

    #[derive(FromAttributesInception, Debug, Default)]
    pub struct Bae {
        pub skip: Option<()>,
    }
}
