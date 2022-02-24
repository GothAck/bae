mod util;

use bae::{FromAttributes, SpannedValue};
use quote::quote;

use self::util::parse_attrs;

#[derive(FromAttributes, Debug)]
struct MyAttr {
    str: SpannedValue<String>,
    optional_int_given: Option<SpannedValue<u16>>,
    optional_int_ignored: Option<SpannedValue<u32>>,
}

#[test]
fn test_spanned_value() {
    let tokens = quote! { #[my_attr(str = "123", optional_int_given = 456)] };

    let attrs = parse_attrs(tokens).unwrap();
    let attr = MyAttr::from_attributes(&attrs).unwrap();

    let optional_int_given = attr.optional_int_given.as_ref().unwrap();

    assert_eq!(attr.str.as_ref(), "123");
    assert_eq!(optional_int_given.to_string(), "456");
    assert!(attr.optional_int_ignored.is_none());
}
