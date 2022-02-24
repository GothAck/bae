mod util;

use bae::{FnCallFixed, FnCallVarArgs, FromAttributes};
use quote::{quote, ToTokens};
use syn::{Expr, ExprCall, ExprLit, ExprPath, Lit, LitInt, LitStr, Path};

use self::util::parse_attrs;

#[derive(FromAttributes, Debug)]
struct MyAttrFixed {
    fn_call_fixed: FnCallFixed<(ExprPath, LitStr, LitInt)>,
    optional_fn_call_fixed: Option<FnCallFixed<(ExprPath, LitStr, LitInt)>>,
    fn_call_fixed_optional_arg: FnCallFixed<(Option<ExprPath>, LitStr, Option<u64>)>,
}

#[derive(FromAttributes, Debug)]
struct MyAttrVarArgs {
    fn_call_var_args: FnCallVarArgs<ExprPath>,
    optional_fn_call_var_args: Option<FnCallVarArgs<LitStr>>,
    fn_call_var_args_optional_arg: FnCallVarArgs<Option<u64>>,

    fn_call_var_args_expr: FnCallVarArgs<Expr>,
}

#[test]
fn test_basic() {
    let tokens = quote! {
    #[my_attr_fixed(
        fn_call_fixed(::my::path, "123", 456), fn_call_fixed_optional_arg(None, "Test", Some(42))
    )] };

    let attrs = parse_attrs(tokens).unwrap();
    let my_attr_fixed = MyAttrFixed::from_attributes(&attrs).unwrap();

    let fn_call_fixed = my_attr_fixed.fn_call_fixed.as_ref();
    assert_eq!(
        fn_call_fixed.0.to_token_stream().to_string(),
        ":: my :: path"
    );
    assert_eq!(fn_call_fixed.1.value(), "123");
    assert_eq!(fn_call_fixed.2.to_string(), "456");

    assert!(my_attr_fixed.optional_fn_call_fixed.is_none());

    let fn_call_fixed_optional_arg = my_attr_fixed.fn_call_fixed_optional_arg.as_ref();
    assert!(fn_call_fixed_optional_arg.0.is_none());
    assert_eq!(fn_call_fixed_optional_arg.1.value(), "Test");
    assert_eq!(fn_call_fixed_optional_arg.2, Some(42));

    let tokens = quote! {
    #[my_attr_var_args(
        fn_call_var_args(::my::path, another::Path, yes, ::std::fs::File),
        fn_call_var_args_optional_arg(Some(999), None, Some(12345), Some(999999999)),
        fn_call_var_args_expr(hello_world(123), test, doot::doooot, 999, 3.141),
    )] };

    let attrs = parse_attrs(tokens).unwrap();
    let my_attr_var_args = MyAttrVarArgs::from_attributes(&attrs).unwrap();

    let fn_call_var_args = my_attr_var_args.fn_call_var_args.as_ref();
    assert_eq!(fn_call_var_args.len(), 4);
    assert_eq!(
        fn_call_var_args[0].to_token_stream().to_string(),
        ":: my :: path"
    );
    assert_eq!(
        fn_call_var_args[1].to_token_stream().to_string(),
        "another :: Path"
    );
    assert_eq!(fn_call_var_args[2].to_token_stream().to_string(), "yes");
    assert_eq!(
        fn_call_var_args[3].to_token_stream().to_string(),
        ":: std :: fs :: File"
    );

    assert!(my_attr_var_args.optional_fn_call_var_args.is_none());

    let fn_call_var_args_optional_arg = my_attr_var_args.fn_call_var_args_optional_arg.as_ref();
    assert_eq!(fn_call_var_args_optional_arg.len(), 4);
    assert_eq!(fn_call_var_args_optional_arg[0], Some(999));
    assert_eq!(fn_call_var_args_optional_arg[1], None);
    assert_eq!(fn_call_var_args_optional_arg[2], Some(12345));
    assert_eq!(fn_call_var_args_optional_arg[3], Some(999999999));

    let fn_call_var_args_expr = my_attr_var_args.fn_call_var_args_expr.as_ref();
    assert_eq!(fn_call_var_args_expr.len(), 5);

    assert!(matches!(fn_call_var_args_expr[0], Expr::Call(..)));
    if let Expr::Call(ExprCall { func, args, .. }) = &fn_call_var_args_expr[0] {
        let func = func.as_ref();
        assert!(matches!(func, Expr::Path(..)));
        if let Expr::Path(ExprPath {
            path: Path { segments, .. },
            ..
        }) = func
        {
            assert_eq!(segments.len(), 1);
            assert_eq!(segments[0].ident, "hello_world");
        }

        assert_eq!(args.len(), 1);
        assert!(matches!(
            args[0],
            Expr::Lit(ExprLit {
                lit: Lit::Int(..),
                ..
            })
        ));
        if let Expr::Lit(ExprLit {
            lit: Lit::Int(lit_int),
            ..
        }) = &args[0]
        {
            assert_eq!(lit_int.to_string(), "123");
        }
    }

    assert!(matches!(fn_call_var_args_expr[1], Expr::Path(..)));
    if let Expr::Path(ExprPath {
        path: Path { segments, .. },
        ..
    }) = &fn_call_var_args_expr[0]
    {
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].ident, "test");
    }

    assert!(matches!(fn_call_var_args_expr[2], Expr::Path(..)));
    if let Expr::Path(ExprPath {
        path: Path { segments, .. },
        ..
    }) = &fn_call_var_args_expr[0]
    {
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].ident, "doot");
        assert_eq!(segments[1].ident, "doooot");
    }

    assert!(matches!(
        fn_call_var_args_expr[3],
        Expr::Lit(ExprLit {
            lit: Lit::Int(..),
            ..
        })
    ));
    if let Expr::Lit(ExprLit {
        lit: Lit::Int(lit_int),
        ..
    }) = &fn_call_var_args_expr[3]
    {
        assert_eq!(lit_int.to_string(), "999");
    }

    assert!(matches!(
        fn_call_var_args_expr[4],
        Expr::Lit(ExprLit {
            lit: Lit::Float(..),
            ..
        })
    ));
    if let Expr::Lit(ExprLit {
        lit: Lit::Float(lit_float),
        ..
    }) = &fn_call_var_args_expr[4]
    {
        assert_eq!(lit_float.to_string(), "3.141");
    }
}
