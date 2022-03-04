use bae::{test_utils::parse_attrs, FromAttributes};
use proc_macro2::Span;
use quote::quote;
use syn::{LitInt, LitStr};

#[derive(FromAttributes)]
#[bae(name = test_attr)]
struct MyAttr {
    normal: LitStr,
    #[bae(name = rename_test)]
    renamed: LitInt,
    #[bae(skip)]
    skipped_switch: Option<()>,
    #[bae(skip, default = skipped_non_default_able_default)]
    skipped_non_default_able: NotDefaultAble,
    #[bae(default = defaulted_default)]
    defaulted_given: Option<LitStr>,
    #[bae(default = defaulted_default)]
    defaulted_ignored: Option<LitStr>,
}

#[derive(Debug, PartialEq, Eq)]
enum NotDefaultAble {
    #[allow(dead_code)]
    One,
    Two,
}

fn skipped_non_default_able_default() -> NotDefaultAble {
    NotDefaultAble::Two
}

fn defaulted_default() -> Option<LitStr> {
    Some(LitStr::new("default", Span::call_site()))
}

#[test]
fn test() {
    let tokens =
        quote! { #[test_attr(normal = "123", rename_test = 456, defaulted_given = "789")] };

    let attrs = parse_attrs(tokens).unwrap();
    let attr = MyAttr::from_attributes(&attrs).unwrap();

    assert_eq!(attr.normal.value(), "123");
    assert_eq!(attr.renamed.to_string(), "456");
    assert_eq!(attr.skipped_switch, None);
    assert_eq!(attr.skipped_non_default_able, NotDefaultAble::Two);
    assert_eq!(attr.defaulted_given.unwrap().value(), "789");
    assert_eq!(attr.defaulted_ignored.unwrap().value(), "default");
}
