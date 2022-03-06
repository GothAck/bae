//! Some test utilities for `bae`

use proc_macro2::{Span, TokenStream};
use syn::{
    parse::{ParseStream, Parser},
    Attribute, Result,
};

use crate::{BaeParse, BaeParseCtx, BaeParseResult};

fn attrs_parser(input: ParseStream) -> Result<Vec<Attribute>> {
    Attribute::parse_outer(input)
}

/// Parse a [`TokenStream`] into a `Vec` of [`syn::Attribute`]
pub fn parse_attrs(tokens: TokenStream) -> Result<Vec<Attribute>> {
    attrs_parser.parse2(tokens)
}

/// Parse a `&str` into a `Vec` of [`syn::Attribute`]
pub fn parse_attrs_str(s: &str) -> Result<Vec<Attribute>> {
    attrs_parser.parse_str(s)
}

#[cfg(feature = "span-locations")]
/// Take a parsed `&str` and slice it using [`Span::start()`] and [`Span::end()`],
/// return the [`Span`]ned slice of the string
pub fn slice_str_from_span(s: &str, span: Span) -> &str {
    let start = span.start();
    let end = span.end();
    assert_eq!(start.line, 1, "Only support single line attrs");
    assert_eq!(end.line, 1, "Only support single line attrs");

    &s[start.column..end.column]
}

/// Calls [`syn::parse::Parser::parse2`] invoking `parser` (a `[BaeParse]::parse*` method), unwrapping the returned [`crate::BaeSpanned`].
pub fn bae_parse2_ctx<T, P>(tokens: TokenStream, ctx: &BaeParseCtx, parser: P) -> Result<T>
where
    T: BaeParse,
    P: FnOnce(ParseStream, &BaeParseCtx) -> BaeParseResult<T>,
{
    let spanned = (|input: ParseStream| parser(input, ctx))
        .parse2(tokens)?;

    Ok(spanned.unwrap())
}

/// Calls [`syn::parse::Parser::parse2`] invoking `parser` (a `[BaeParse]::parse*` method), unwrapping the returned [`crate::BaeSpanned`].
pub fn bae_parse2<T, P>(tokens: TokenStream, parser: P) -> Result<T>
where
    T: BaeParse,
    P: FnOnce(ParseStream, &BaeParseCtx) -> BaeParseResult<T>,
{
    let ctx = BaeParseCtx::new(Span::call_site());
    bae_parse2_ctx(tokens, &ctx, parser)
}
