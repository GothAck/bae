use proc_macro2::TokenStream;
use syn::{
    parse::{ParseStream, Parser},
    Attribute, Result,
};

fn attrs_parser(input: ParseStream) -> Result<Vec<Attribute>> {
    Attribute::parse_outer(&input)
}

pub fn parse_attrs(tokens: TokenStream) -> Result<Vec<Attribute>> {
    attrs_parser.parse2(tokens)
}
