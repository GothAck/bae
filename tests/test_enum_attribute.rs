use bae::{test_utils::parse_attrs, EnumAttribute, FromAttributes};
use quote::quote;

#[derive(EnumAttribute, Debug)]
enum MyEnumAttribute {
    First,
    Second,
    #[bae(skip)]
    #[allow(dead_code)]
    Skipped,
}

#[derive(FromAttributes, Debug)]
struct MyAttr {
    my_enum: MyEnumAttribute,
}

#[test]
fn test() {
    let tokens = quote! { #[my_attr(my_enum = First)] };

    let attrs = parse_attrs(tokens).unwrap();
    let attr = MyAttr::from_attributes(&attrs).unwrap();

    assert!(matches!(attr.my_enum, MyEnumAttribute::First));
}

#[test]
fn test_skipped() {
    let tokens = quote! { #[my_attr(my_enum = Skipped)] };

    let attrs = parse_attrs(tokens).unwrap();
    let err = MyAttr::from_attributes(&attrs).unwrap_err();

    assert_eq!(
        err.to_string(),
        "Invalid value, supported values are `First`, `Second`"
    );
}
