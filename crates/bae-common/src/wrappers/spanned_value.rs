use std::{fmt, ops::Deref};

use proc_macro2::Span;
use syn::parse::ParseStream;

use crate::{
    types_support::{BaeSupportedAllType, BaeSupportedOtherType, BaeSupportedTypeBunked},
    BaeDefault, BaeDefaultedValue, BaeParse, BaeParseCtx, BaeParseResult,
};

/// Wrap a type in this (at the root of the type hiererchy), and you get a deref-able and as_ref-able
/// container on which you can also call [`SpannedValue::span()`] to retrieve the [`Span`] of the contained value.
///
/// This is especially useful with the [`BaeParse`] impls for String, integers and other non-`syn` types
/// that do not store [`Span`]s.
///
/// Interaction with `Option<T>`:
/// It's best to use `Option<SpannedValue<T>>`, if you use `SpannedValue<Option<T>>` things will still work, but
/// the outer `SpannedValue` will have it's [`Span`]s set to [`Span::call_site()`] for missing values.
#[derive(Clone)]
pub struct SpannedValue<T>
where
    T: BaeParse,
{
    inner: T,
    span: Span,
    key_span: Span,
}

impl<T> fmt::Debug for SpannedValue<T>
where
    T: BaeParse + fmt::Debug,
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
    T: BaeParse,
{
    fn new(inner: T, span: Span, key_span: Span) -> Self {
        Self {
            inner,
            span,
            key_span,
        }
    }

    /// Get the [`Span`] associated with the parsed content, if applicable.
    pub fn span(&self) -> Span {
        self.span
    }

    /// Get the [`Span`] assiciated with this [`SpannedValue`]'s key, if applicable.
    pub fn key_span(&self) -> Span {
        self.key_span
    }
}

impl<'a, T> Deref for SpannedValue<T>
where
    Self: 'a,
    T: BaeParse,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> BaeParse for SpannedValue<T>
where
    T: BaeParse,
{
    fn parse(input: ParseStream, ctx: &BaeParseCtx) -> BaeParseResult<Self> {
        Ok(<T as BaeParse>::parse(input, ctx)?
            .map_with_span(|inner, span| Self::new(inner, span, ctx.attr_ident_span())))
    }

    fn parse_prefix(input: ParseStream, ctx: &BaeParseCtx) -> BaeParseResult<Self> {
        Ok(<T as BaeParse>::parse_prefix(input, ctx)?
            .map_with_span(|inner, span| Self::new(inner, span, ctx.attr_ident_span())))
    }

    fn parse_fn_arg(input: ParseStream, ctx: &BaeParseCtx) -> BaeParseResult<Self> {
        Ok(<T as BaeParse>::parse_fn_arg(input, ctx)?
            .map_with_span(|inner, span| Self::new(inner, span, ctx.attr_ident_span())))
    }
}

impl<T> BaeDefault for SpannedValue<T>
where
    T: BaeParse + BaeDefault,
{
    fn bae_default() -> BaeDefaultedValue<Self> {
        use BaeDefaultedValue::*;

        let inner = if let Default(inner) = <T as BaeDefault>::bae_default() {
            inner
        } else {
            return NoDefault;
        };

        Default(Self::new(inner, Span::call_site(), Span::call_site())) // FIXME: call_site()
    }
}

impl<T> BaeSupportedOtherType for SpannedValue<T> where T: BaeParse {}
impl<T> BaeSupportedAllType for SpannedValue<T> where T: BaeParse {}
impl<T> BaeSupportedTypeBunked for SpannedValue<T> where T: BaeParse {}
