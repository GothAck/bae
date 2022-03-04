use bae::{test_utils::parse_attrs, FromAttributes};
use quote::quote;
use syn::{LitInt, LitStr};

#[derive(FromAttributes, Debug)]
struct MyAttr(
    LitStr,
    Option<LitInt>,
    Option<LitInt>,
    Option<()>,
    Option<()>,
);

fn main() {
    let tokens = quote! { #[my_attr(str = "123", optional_int_given = 456, switch_given)] };

    let attrs = parse_attrs(tokens).unwrap();
    let attr = MyAttr::from_attributes(&attrs).unwrap();

    assert_eq!(attr.0.value(), "123");
    assert_eq!(attr.1.unwrap().to_string(), "456");
    assert_eq!(attr.2, None);
    assert!(attr.3.is_some());
    assert!(attr.4.is_none());
}
