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
pub mod types_support;
mod wrappers;

pub use self::{
    default::{BaeDefault, BaeDefaultedValue},
    parse::{BaeParse, BaeParseVia},
    wrappers::{
        fncall::{FnCallFixed, FnCallVarArgs},
        spanned_value::SpannedValue,
    },
};
