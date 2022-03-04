//! Common utilities and types for [`bae`](https://crates.io/crates/bae)

#![doc(html_root_url = "https://docs.rs/bae/0.1.7")]
#![allow(clippy::let_and_return)]
#![deny(
    unused_variables,
    mutable_borrow_reservation_conflict,
    dead_code,
    unused_must_use,
    unused_imports,
    missing_docs
)]

mod default;
#[doc(hidden)]
pub mod from_attributes_meta;
mod parse;
#[doc(hidden)]
pub mod private;

pub mod types_support;
mod wrappers;

pub use self::{
    default::{BaeDefault, BaeDefaultedValue},
    parse::{BaeParse, BaeParseResult, BaeParseVia, BaeSpanned},
    wrappers::{
        fncall::{FnCallFixed, FnCallVarArgs},
        spanned_value::SpannedValue,
    },
};

/// See root module docs for more info.
pub trait FromAttributes
where
    Self: Sized,
{
    /// Try to parse `syn::Attribute`s into `Self`.
    ///
    /// Returns:
    ///     Err(syn::Error) - on attribute parsing error.
    ///     Ok(None)        - when none of the `syn::Attribute`s match this attribute's name.
    ///     Ok(Some(Self))  - when one of the `syn::Attribute`s match this attribute's name.
    fn try_from_attributes(attrs: &[syn::Attribute]) -> syn::Result<Option<Self>>;

    /// Parse `syn::Attribute`s into `Self`.
    ///
    /// Returns:
    ///     Err(syn::Error) - on attribute parsing error, or none of the `syn::Attribute`s
    ///                       match this attribute's name.
    ///     Ok(Self)        - when one of the `syn::Attribute`s match this attribute's name.
    fn from_attributes(attrs: &[syn::Attribute]) -> syn::Result<Self>;
}
