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

use heck::SnakeCase;
use indexmap::IndexMap;
use proc_macro2::TokenStream;
use proc_macro_error::*;
use quote::*;
use syn::{spanned::Spanned, *};

/// See root module docs for more info.
#[proc_macro_derive(FromAttributes, attributes())]
#[proc_macro_error]
pub fn from_attributes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let item = parse_macro_input!(input as ItemStruct);
    FromAttributes::new(item).expand().into()
}

#[derive(Debug)]
struct FromAttributes {
    item: ItemStruct,
    attr_name: LitStr,
    fields: IndexMap<Ident, FromAttributesField>,
    tokens: TokenStream,
}

impl FromAttributes {
    fn new(item: ItemStruct) -> Self {
        let attr_name = LitStr::new(&item.ident.to_string().to_snake_case(), item.ident.span());
        let fields = item
            .fields
            .iter()
            .map(|field| {
                let field = FromAttributesField::new(field, &attr_name);
                (field.ident.clone(), field)
            })
            .collect();

        Self {
            item,
            attr_name,
            fields,
            tokens: TokenStream::new(),
        }
    }

    fn expand(mut self) -> TokenStream {
        self.expand_from_attributes_method();
        self.expand_parse_impl();

        if std::env::var("BAE_DEBUG").is_ok() {
            eprintln!("{}", self.tokens);
        }

        self.tokens
    }

    fn struct_name(&self) -> &Ident {
        &self.item.ident
    }

    fn expand_from_attributes_method(&mut self) {
        let struct_name = self.struct_name();
        let attr_name = &self.attr_name;

        let code = quote! {
            impl #struct_name {
                pub fn try_from_attributes(attrs: &[syn::Attribute]) -> syn::Result<Option<Self>> {
                    use syn::spanned::Spanned;

                    for attr in attrs {
                        match attr.path.get_ident() {
                            Some(ident) if ident == #attr_name => {
                                return Some(syn::parse2::<Self>(attr.tokens.clone())).transpose()
                            }
                            // Ignore other attributes
                            _ => {},
                        }
                    }

                    Ok(None)
                }

                pub fn from_attributes(attrs: &[syn::Attribute]) -> syn::Result<Self> {
                    if let Some(attr) = Self::try_from_attributes(attrs)? {
                        Ok(attr)
                    } else {
                        Err(syn::Error::new(
                            proc_macro2::Span::call_site(),
                            &format!("missing attribute `#[{}]`", #attr_name),
                        ))
                    }
                }
            }
        };
        self.tokens.extend(code);
    }

    fn expand_parse_impl(&mut self) {
        let struct_name = self.struct_name();
        let attr_name = &self.attr_name;

        let variable_declarations = self
            .fields
            .values()
            .map(FromAttributesField::expand_variable_decl);

        let match_arms = self
            .fields
            .values()
            .map(FromAttributesField::expand_match_arms);

        let unwrap_mandatory_fields = self
            .fields
            .values()
            .filter_map(FromAttributesField::expand_unwrap_mandatory_field);

        let set_fields = self
            .fields
            .values()
            .map(FromAttributesField::expand_set_field);

        let mut supported_args = self
            .fields
            .keys()
            .map(|field_name| format!("`{}`", field_name))
            .collect::<Vec<_>>();
        supported_args.sort_unstable();
        let supported_args = supported_args.join(", ");

        let code = quote! {
            impl syn::parse::Parse for #struct_name {
                #[allow(unreachable_code, unused_imports, unused_variables)]
                fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
                    #(#variable_declarations)*

                    let content;
                    syn::parenthesized!(content in input);
                    let content_span = content.span();

                    while !content.is_empty() {
                        let bae_attr_ident = content.parse::<syn::Ident>()?;

                        match &*bae_attr_ident.to_string() {
                            #(#match_arms)*
                            other => {
                                return syn::Result::Err(
                                    syn::Error::new(
                                        bae_attr_ident.span(),
                                        &format!(
                                            "`#[{}]` got unknown `{}` argument. Supported arguments are {}",
                                            #attr_name,
                                            other,
                                            #supported_args,
                                        ),
                                    )
                                );
                            }
                        }

                        content.parse::<syn::Token![,]>().ok();
                    }

                    #(#unwrap_mandatory_fields)*

                    syn::Result::Ok(Self { #(#set_fields)* })
                }
            }
        };
        self.tokens.extend(code);
    }
}

#[derive(Debug)]
struct FromAttributesField {
    field: Field,
    attr_name: LitStr,
    ident: Ident,
}

impl FromAttributesField {
    fn new(field: &Field, attr_name: &LitStr) -> Self {
        let ident = field
            .ident
            .clone()
            .unwrap_or_else(|| abort!(field.span(), "Field without a name"));

        Self {
            field: field.clone(),
            attr_name: attr_name.clone(),
            ident,
        }
    }

    fn expand_variable_decl(&self) -> TokenStream {
        let name = &self.ident;
        let ty = &self.field.ty;
        quote! { let mut #name: Option<#ty> = std::option::Option::None; }
    }

    fn expand_match_arms(&self) -> TokenStream {
        let field_name = &self.ident;
        let ty = &self.field.ty;
        let pattern = LitStr::new(&field_name.to_string(), self.field.span());

        return quote! {
            #pattern => {
                #field_name = Some(<#ty as ::bae::BaeParse>::parse_prefix(&content)?);
            }
        };
    }

    fn expand_unwrap_mandatory_field(&self) -> Option<TokenStream> {
        let attr_name = &self.attr_name;
        let field_name = &self.ident;
        let arg_name = LitStr::new(&field_name.to_string(), self.field.span());

        Some(quote! {
            let #field_name = #field_name
                .map(|v| ::bae::BaeDefaultedValue::Present(v))
                .unwrap_or_else(<_ as ::bae::BaeDefault>::bae_default)
                .ok_or_syn_error(
                    content_span,
                    &format!("`#[{}]` is missing `{}` argument", #attr_name, #arg_name),
                )?;
        })
    }

    fn expand_set_field(&self) -> TokenStream {
        let field_name = &self.ident;
        quote! { #field_name, }
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
