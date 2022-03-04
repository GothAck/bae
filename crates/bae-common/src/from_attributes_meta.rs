extern crate proc_macro;

use heck::ToSnakeCase;
use indexmap::IndexMap;
use proc_macro2::TokenStream;
use quote::*;
use syn::{spanned::Spanned, *};

pub trait FromAttributesData
where
    Self: Sized,
{
    fn new(attrs: &[Attribute]) -> Result<Self>;
    fn rename_attr_name(&self, original: String) -> String;
}

pub trait FromAttributesFieldData
where
    Self: Sized,
{
    fn new(attrs: &[Attribute]) -> Result<Self>;
    fn rename_field_name(&self, original: String) -> String;
    fn skip(&self) -> bool;
    fn default(&self) -> Option<TokenStream>;
}

#[derive(Debug)]
pub struct FromAttributesMeta<
    Data: FromAttributesData,
    FieldData: FromAttributesFieldData,
    const COMMON: bool,
> {
    item: ItemStruct,
    bae_path: Path,
    #[allow(dead_code)] // FIXME: remove dead_code
    data: Data,
    attr_name: LitStr,
    fields: IndexMap<Ident, FromAttributesFieldMeta<FieldData>>,
    tokens: TokenStream,
}

impl<Data: FromAttributesData, FieldData: FromAttributesFieldData, const COMMON: bool>
    FromAttributesMeta<Data, FieldData, COMMON>
{
    pub fn new(item: ItemStruct) -> Result<Self> {
        let bae_path = {
            if COMMON {
                parse_str("::bae_common")?
            } else {
                parse_str("::bae")?
            }
        };
        let data = Data::new(&item.attrs)?;
        let attr_name = {
            let attr_name = item.ident.to_string().to_snake_case();
            let attr_name = data.rename_attr_name(attr_name);
            LitStr::new(attr_name.as_str(), item.ident.span())
        };
        let fields = item
            .fields
            .iter()
            .map(|field| {
                let field = FromAttributesFieldMeta::new(field, &attr_name, &bae_path)?;
                Ok((field.ident.clone(), field))
            })
            .collect::<Result<IndexMap<_, _>>>()?;

        Ok(Self {
            item,
            bae_path,
            data,
            attr_name,
            fields,
            tokens: TokenStream::new(),
        })
    }

    pub fn expand(mut self) -> TokenStream {
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
        let bae_path = &self.bae_path;
        let attr_name = &self.attr_name;

        let code = quote! {
            impl #bae_path::FromAttributes for #struct_name {
                fn try_from_attributes(attrs: &[syn::Attribute]) -> syn::Result<Option<Self>> {
                    use ::syn::spanned::Spanned;
                    use ::proc_macro2::Span;
                    use #bae_path::private::IterCombineSynErrors;

                    let attrs = attrs
                        .iter()
                        .filter_map(|attr| match attr.path.get_ident() {
                            Some(ident) if ident == #attr_name => {
                                Some(
                                    ::syn::parse2::<Self>(attr.tokens.clone())
                                        .map(|parsed| (parsed, attr.span()))
                                )
                            },
                            _ => None,
                        })
                        .collect_syn_error::<Vec<_>>()?;

                    attrs
                        .into_iter()
                        .fold(Ok(None), |accum, (attr, span)| {
                            let error_new = || syn::Error::new(
                                span,
                                &format!("duplicate attribute `#[{}]`", #attr_name),
                            );

                            match accum {
                                Ok(None) => Ok(Some(attr)),
                                Ok(Some(..)) => Err(error_new()),
                                Err(mut error) => {
                                    error.combine(error_new());
                                    Err(error)
                                }
                            }
                        })
                }

                fn from_attributes(attrs: &[syn::Attribute]) -> syn::Result<Self> {
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
            .map(FromAttributesFieldMeta::expand_variable_decl);

        let match_arms = self
            .fields
            .values()
            .map(FromAttributesFieldMeta::expand_match_arms);

        let unwrap_mandatory_fields = self
            .fields
            .values()
            .map(FromAttributesFieldMeta::expand_unwrap_mandatory_field);

        let set_fields = self
            .fields
            .values()
            .map(FromAttributesFieldMeta::expand_set_field);

        let mut supported_args = self
            .fields
            .keys()
            .map(|ident| format!("`{}`", ident))
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
struct FromAttributesFieldMeta<FieldData: FromAttributesFieldData> {
    field: Field,
    attr_name: LitStr,
    bae_path: Path,
    data: FieldData,
    ident: Ident,
    variable_ident: Ident,
    field_name: LitStr,
}

impl<FieldData: FromAttributesFieldData> FromAttributesFieldMeta<FieldData> {
    fn new(field: &Field, attr_name: &LitStr, bae_path: &Path) -> Result<Self> {
        let data = FieldData::new(&field.attrs)?;

        let ident = field
            .ident
            .clone()
            .ok_or_else(|| Error::new(field.span(), "Field without a name"))?;

        let variable_ident = format_ident!("___{}", ident);

        let field_name = {
            let field_name = ident.to_string();
            let field_name = data.rename_field_name(field_name);

            LitStr::new(&field_name, ident.span())
        };

        Ok(Self {
            field: field.clone(),
            attr_name: attr_name.clone(),
            bae_path: bae_path.clone(),
            data,
            ident,
            variable_ident,
            field_name,
        })
    }

    fn expand_variable_decl(&self) -> Option<TokenStream> {
        if self.data.skip() {
            return None;
        }

        let variable_ident = &self.variable_ident;
        let ty = &self.field.ty;
        Some(quote! { let mut #variable_ident: Option<#ty> = std::option::Option::None; })
    }

    fn expand_match_arms(&self) -> Option<TokenStream> {
        if self.data.skip() {
            return None;
        }

        let variable_ident = &self.variable_ident;
        let pattern = &self.field_name;
        let attr_name = &self.attr_name;
        let field_name = &self.field_name;
        let bae_path = &self.bae_path;

        Some(quote! {
            #pattern => {
                if #variable_ident.is_some() {
                    return Err(::syn::Error::new(
                        content_span,
                        &format!("`#[{}]` argument `{}` specified multiple times", #attr_name, #field_name),
                    ));
                }

                #variable_ident = Some(<_ as #bae_path::BaeParse>::parse_prefix(&content)?);
            }
        })
    }

    fn expand_unwrap_mandatory_field(&self) -> TokenStream {
        let variable_ident = &self.variable_ident;

        if self.data.skip() {
            let ty = &self.field.ty;

            let default = self
                .data
                .default()
                .unwrap_or_else(|| quote! { ::std::default::Default::default });

            quote! {
                let #variable_ident: #ty = #default();
            }
        } else {
            let bae_path = &self.bae_path;
            let attr_name = &self.attr_name;
            let field_name = &self.field_name;

            let default = self
                .data
                .default()
                .map(|default| quote! { #bae_path::BaeDefaultedValue::Default(#default()) })
                .unwrap_or_else(|| quote! { <_ as #bae_path::BaeDefault>::bae_default() });

            quote! {
                let #variable_ident = #variable_ident
                    .map(|v| #bae_path::BaeDefaultedValue::Present(v))
                    .unwrap_or_else(|| #default)
                    .ok_or_syn_error(
                        content_span,
                        &format!("`#[{}]` is missing `{}` argument", #attr_name, #field_name),
                    )?;
            }
        }
    }

    fn expand_set_field(&self) -> TokenStream {
        let ident = &self.ident;
        let variable_ident = &self.variable_ident;

        quote! { #ident: #variable_ident, }
    }
}
