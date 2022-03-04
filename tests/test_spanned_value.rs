#![cfg(feature = "span-locations")]

use bae::{
    test_utils::{parse_attrs_str, slice_str_from_span},
    FnCallFixed, FromAttributes, SpannedValue,
};
use quote::ToTokens;
use syn::{ExprPath, LitInt, LitStr};

#[derive(FromAttributes, Debug)]
struct MyAttr {
    string: SpannedValue<String>,

    optional_int_given: Option<SpannedValue<u16>>,
    optional_int_ignored: Option<SpannedValue<u32>>,

    optional_fn_call_fixed: Option<SpannedValue<FnCallFixed<(ExprPath, LitStr, LitInt)>>>,
}

#[test]
fn test_spanned_value() {
    let s = "#[my_attr(string = \"123\", optional_int_given = 456, optional_fn_call_fixed(::syn::parse, \"789\", 012))]";

    let attrs = parse_attrs_str(s).unwrap();
    let attr = MyAttr::from_attributes(&attrs).unwrap();

    let optional_int_given = attr.optional_int_given.as_ref().unwrap();
    let optional_fn_call_fixed = attr.optional_fn_call_fixed.as_ref().unwrap();

    assert_eq!(*attr.string, "123");
    assert_eq!(optional_int_given.to_string(), "456");
    assert!(attr.optional_int_ignored.is_none());

    assert_eq!(
        optional_fn_call_fixed.0.to_token_stream().to_string(),
        ":: syn :: parse"
    );
    assert_eq!(optional_fn_call_fixed.1.value(), "789");
    assert_eq!(optional_fn_call_fixed.2.base10_digits(), "12");

    assert_eq!(slice_str_from_span(s, attr.string.key_span()), "string",);
    assert_eq!(slice_str_from_span(s, attr.string.span()), "\"123\"",);

    assert_eq!(
        slice_str_from_span(s, optional_int_given.key_span()),
        "optional_int_given",
    );
    assert_eq!(slice_str_from_span(s, optional_int_given.span()), "456",);

    assert_eq!(
        slice_str_from_span(s, optional_fn_call_fixed.key_span()),
        "optional_fn_call_fixed",
    );
    assert_eq!(
        slice_str_from_span(s, optional_fn_call_fixed.span()),
        "::syn::parse, \"789\", 012",
    );
}
