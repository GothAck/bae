#[cfg(feature = "span-locations")]
use proc_macro2::Span;
use proc_macro2::TokenStream;
use syn::{
    parse::{ParseStream, Parser},
    Attribute, Result,
};

fn attrs_parser(input: ParseStream) -> Result<Vec<Attribute>> {
    Attribute::parse_outer(&input)
}

#[allow(dead_code)]
pub fn parse_attrs(tokens: TokenStream) -> Result<Vec<Attribute>> {
    attrs_parser.parse2(tokens)
}

#[allow(dead_code)]
pub fn parse_attrs_str(s: &str) -> Result<Vec<Attribute>> {
    attrs_parser.parse_str(s)
}

#[cfg(feature = "span-locations")]
#[allow(dead_code)]
pub fn slice_str_from_span(s: &str, span: Span) -> &str {
    let start = span.start();
    let end = span.end();
    assert_eq!(start.line, 1, "Only support single line attrs");
    assert_eq!(end.line, 1, "Only support single line attrs");

    &s[start.column..end.column]
}
