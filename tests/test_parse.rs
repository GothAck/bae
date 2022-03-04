use bae::{test_utils::parse_attrs, FromAttributes};
use quote::{quote, ToTokens};
use syn::{ExprCall, ExprPath};

#[derive(FromAttributes, Debug)]
struct MyAttr {
    string: String,
    optional_string: Option<String>,
    num_u8: u8,
    num_u16: u16,
    num_u32: u32,
    num_u64: u64,
    num_u128: u128,
    num_usize: usize,
    optional_num_f32: Option<f32>,
    expr_path: ExprPath,
    expr_call: ExprCall,
}

#[test]
fn test_basic() {
    let tokens = quote! {
    #[my_attr(
        string = "012",
        num_u8 = 255,
        num_u16 = 16535,
        num_u32 = 999999,
        num_u64 = 9999999,
        num_u128 = 99999999,
        num_usize = 99999999usize,
        optional_num_f32 = 3.141,
        expr_path = ::std::default::Default,
        expr_call = my_function(my_arg)
    )] };

    let attrs = parse_attrs(tokens).unwrap();
    let attr = MyAttr::from_attributes(&attrs).unwrap();

    assert_eq!(attr.string, "012");
    assert_eq!(attr.optional_string, None);
    assert_eq!(attr.num_u8, 255);
    assert_eq!(attr.num_u16, 16535);
    assert_eq!(attr.num_u32, 999999);
    assert_eq!(attr.num_u64, 9999999);
    assert_eq!(attr.num_u128, 99999999);
    assert_eq!(attr.num_usize, 99999999);
    assert_eq!(attr.optional_num_f32, Some(3.141));
    assert_eq!(
        attr.expr_path.path.to_token_stream().to_string(),
        ":: std :: default :: Default"
    );
    assert_eq!(
        attr.expr_call.to_token_stream().to_string(),
        "my_function (my_arg)"
    );
}
