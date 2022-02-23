mod util;

use bae::FromAttributes;
use quote::quote;
use syn::{LitInt, LitStr};

use self::util::parse_attrs;

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
    let tokens = quote! { #[my_attr(str = "123", optional_int_given = 456, switch_given)] };

    let attrs = parse_attrs(tokens).unwrap();
    let attr = MyAttr::from_attributes(&attrs).unwrap();

    assert_eq!(attr.str.value(), "123");
    assert_eq!(attr.optional_int_given.unwrap().to_string(), "456");
    assert_eq!(attr.optional_int_ignored, None);
    assert!(attr.switch_given.is_some());
    assert!(attr.switch_ignored.is_none());
}
