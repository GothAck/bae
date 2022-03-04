use std::{fmt, ops::Deref};

use proc_macro2::Span;
use syn::parse::ParseStream;

use crate::{
    parse::{BaeParse, BaeParseVia},
    types_support::{BaeSupportedAllType, BaeSupportedOtherType, BaeSupportedTypeBunked},
    BaeDefault, BaeDefaultedValue, BaeParseResult,
};

/// Wrap a type in this (at the root of the type hiererchy), and you get a deref-able and as_ref-able
/// container on which you can also call `.span()` to retrieve the `Span` of the contained value.
///
/// Currently only implemented for types that implement `BaeParseVia`, aka non-`syn` types such
/// as String, integer, and float types.
///
/// This is especially useful with the `BaeParse` impls for String, integers and other non-`syn` types
/// that do not store `span`s.
pub struct SpannedValue<T>
where
    T: BaeParseVia,
{
    inner: T,
    span: Option<Span>,
}

impl<T> fmt::Debug for SpannedValue<T>
where
    T: BaeParseVia + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("SpannedValue")
            .field("inner", &self.inner)
            .field("span", &self.span)
            .finish()
    }
}

impl<T> SpannedValue<T>
where
    T: BaeParseVia,
    <T as BaeParseVia>::Via: BaeParse,
{
    fn new(inner: T, span: Option<Span>) -> Self {
        Self { inner, span }
    }

    /// Get the `Span` associated with the parsed content, if applicable.
    pub fn span(&self) -> Option<Span> {
        self.span
    }
}

impl<'a, T> Deref for SpannedValue<T>
where
    Self: 'a,
    T: BaeParseVia,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> AsRef<T> for SpannedValue<T>
where
    T: BaeParseVia,
{
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> BaeParse for SpannedValue<T>
where
    T: BaeParseVia + BaeParse,
    <T as BaeParseVia>::Via: BaeParse,
{
    fn parse(input: ParseStream) -> BaeParseResult<Self> {
        Ok(<T as BaeParse>::parse(input)?.map_with_span(Self::new))
    }

    fn parse_prefix(input: ParseStream) -> BaeParseResult<Self> {
        Ok(<T as BaeParse>::parse_prefix(input)?.map_with_span(Self::new))
    }

    fn parse_fn_arg(input: ParseStream) -> BaeParseResult<Self> {
        Ok(<T as BaeParse>::parse_fn_arg(input)?.map_with_span(Self::new))
    }
}

impl<T> BaeDefault for SpannedValue<T>
where
    T: BaeParseVia + BaeDefault,
    <T as BaeParseVia>::Via: BaeParse,
{
    fn bae_default() -> BaeDefaultedValue<Self> {
        use BaeDefaultedValue::*;

        let inner = if let Default(inner) = <T as BaeDefault>::bae_default() {
            inner
        } else {
            return NoDefault;
        };

        Default(Self::new(inner, None))
    }
}

impl<T> BaeSupportedOtherType for SpannedValue<T> where T: BaeParseVia {}
impl<T> BaeSupportedAllType for SpannedValue<T> where T: BaeParseVia {}
impl<T> BaeSupportedTypeBunked for SpannedValue<T> where T: BaeParseVia {}
