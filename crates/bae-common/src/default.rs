use std::fmt::Display;

use proc_macro2::Span;
use syn::{Error, Result};

use crate::types_support::BaeSupportedSynType;

/// Specialized version of `std::default::Default` that returns `BaeDefaultedValue`.
///
/// This is due to our special case parsing of:
/// `()`
///     Used in `Option<()>` switches, no default value
/// `Option<T> where T: BaeParse`
///     Used in `Option<()>` switches, default value of `None`
/// `String`, `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128`, `isize`, `f32`, `f64`
///     No default value
pub trait BaeDefault
where
    Self: Sized,
{
    /// Get the default of the implementing type.
    fn bae_default() -> BaeDefaultedValue<Self>;
}

impl BaeDefault for () {
    fn bae_default() -> BaeDefaultedValue<Self> {
        BaeDefaultedValue::NoDefault
    }
}

impl<T> BaeDefault for Option<T>
where
    T: BaeDefault,
{
    fn bae_default() -> BaeDefaultedValue<Self> {
        BaeDefaultedValue::Default(None)
    }
}

impl<T> BaeDefault for T
where
    T: BaeSupportedSynType,
{
    fn bae_default() -> BaeDefaultedValue<Self> {
        BaeDefaultedValue::NoDefault
    }
}

macro_rules! impl_bae_default_no_default {
    ($($x:ty),+) => (
        $(
            impl BaeDefault for $x {
                fn bae_default() -> BaeDefaultedValue<Self> {
                    BaeDefaultedValue::NoDefault
                }
            }
        )+
    );
}

impl_bae_default_no_default!(
    String, u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);

/// Result of a `BaeDefault::bae_default()` call, mappable to `Result<T, syn::Error>` using `.ok_or_syn_error(Span, impl Display)`
pub enum BaeDefaultedValue<T>
where
    T: Sized,
{
    /// Existing value. Maps to `Result::Ok`
    Present(T),
    /// Defaulted value. Maps to `Result::Ok`
    Default(T),
    /// No existing or default value. Maps to `Result::Error`
    NoDefault,
}

impl<T> BaeDefaultedValue<T> {
    /// Transforms the `BaeDefaultResult<T>` into a `Result<T, syn::Error>`,
    /// mapping Some(v) to Ok(v) and None to Err(syn::Error(span, msg)).
    pub fn ok_or_syn_error<U: Display>(self, span: Span, msg: U) -> Result<T> {
        use BaeDefaultedValue::*;
        match self {
            Present(v) | Default(v) => Ok(v),
            NoDefault => Err(Error::new(span, msg)),
        }
    }
}
