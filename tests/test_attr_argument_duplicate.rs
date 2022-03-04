use bae::{test_utils::parse_attrs, FromAttributes};
use quote::quote;
use syn::{LitInt, LitStr};

#[allow(dead_code)]
#[derive(FromAttributes, Debug)]
struct MyAttr {
    str: LitStr,
    optional_int_given: Option<LitInt>,
    optional_int_ignored: Option<LitInt>,
    switch_given: Option<()>,
    switch_ignored: Option<()>,
}

#[test]
fn test() {
    let tokens =
        quote! { #[my_attr(str = "123", optional_int_given = 456, switch_given, str = "789")] };

    let attrs = parse_attrs(tokens).unwrap();
    let err = MyAttr::from_attributes(&attrs).unwrap_err();

    assert_eq!(
        err.to_string(),
        "`#[my_attr]` argument `str` specified multiple times"
    );
}
