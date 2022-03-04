use std::ops::Deref;

use paste::paste;
use proc_macro2::TokenStream;
use syn::{parse::ParseStream, spanned::Spanned, Error, Result};

use crate::{
    types_support::{BaeSupportedAllType, BaeSupportedOtherType, BaeSupportedTypeBunked},
    BaeDefault, BaeDefaultedValue, BaeParse, BaeParseResult, BaeSpanned,
};

#[derive(Debug)]
/// Wrap a tuple of a bunch of types in this to make your attribute argument "callable", with fixed type arguments, instead of assignable.
///
/// For example:
/// ```
/// # use quote::{quote, ToTokens};
/// # use syn::{parse::{Parser, ParseStream}, Attribute, ExprArray, LitStr, Path, Type, ExprTuple};
/// use bae::{FromAttributes, FnCallFixed};
///
/// #[derive(FromAttributes)]
/// struct MyAttr {
///     my_callable: FnCallFixed<(ExprArray, LitStr, Path, Type, ExprTuple)>,
///     other: Option<()>,
/// }
///
/// // MyAttr can now parse the following:
/// # let tokens = quote! {
/// #[my_attr( my_callable([1, 2, 3, 4], "hehehe", ::std::ops::Deref, MyType, (9, "hello", 3.14, "world")), other )]
/// # };
/// #
/// # fn attrs_parser(input: ParseStream) -> syn::Result<Vec<Attribute>> {
/// #     Attribute::parse_outer(&input)
/// # }
/// # let attrs = attrs_parser.parse2(tokens).unwrap();
/// # let attr = MyAttr::from_attributes(&attrs).unwrap();
/// # let my_callable = attr.my_callable.as_ref();
/// # assert_eq!(my_callable.0.to_token_stream().to_string(), "[1 , 2 , 3 , 4]");
/// # assert_eq!(my_callable.1.value(), "hehehe");
/// # assert_eq!(my_callable.2.to_token_stream().to_string(), ":: std :: ops :: Deref");
/// # assert_eq!(my_callable.3.to_token_stream().to_string(), "MyType");
/// # assert_eq!(my_callable.4.to_token_stream().to_string(), "(9 , \"hello\" , 3.14 , \"world\")");
/// # assert!(attr.other.is_some());
/// ```
pub struct FnCallFixed<T>
where
    T: Sized,
{
    inner: T,
}

#[derive(Debug)]

/// Supply one type to this and receive a callable attribute argument with varying argument count.
///
/// For example:
/// ```
/// # use quote::quote;
/// # use syn::{parse::{Parser, ParseStream}, Attribute, ExprArray, LitStr, Path, Type, ExprTuple};
/// use bae::{FromAttributes, FnCallVarArgs};
///
/// #[derive(FromAttributes)]
/// struct MyAttr {
///     my_callable: FnCallVarArgs<LitStr>,
///     other: Option<()>,
/// }
///
/// // MyAttr can now parse the following:
/// # let tokens = quote! {
/// #[my_attr( my_callable("Hello,", " World!", "as many", "arguments", "as", "you", "like", "!"), other )]
/// # };
/// #
/// # fn attrs_parser(input: ParseStream) -> syn::Result<Vec<Attribute>> {
/// #     Attribute::parse_outer(&input)
/// # }
/// # let attrs = attrs_parser.parse2(tokens).unwrap();
/// # let attr = MyAttr::from_attributes(&attrs).unwrap();
/// # assert_eq!(attr.my_callable[0].value(), "Hello,");
/// # assert_eq!(attr.my_callable[1].value(), " World!");
/// # assert_eq!(attr.my_callable[2].value(), "as many");
/// # assert_eq!(attr.my_callable[3].value(), "arguments");
/// # assert_eq!(attr.my_callable[4].value(), "as");
/// # assert_eq!(attr.my_callable[5].value(), "you");
/// # assert_eq!(attr.my_callable[6].value(), "like");
/// # assert_eq!(attr.my_callable[7].value(), "!");
/// # assert!(attr.other.is_some());
/// ```
pub struct FnCallVarArgs<T>
where
    T: Sized,
{
    inner: Vec<T>,
}

impl<T> BaeDefault for FnCallFixed<T>
where
    T: Sized,
{
    fn bae_default() -> BaeDefaultedValue<Self> {
        BaeDefaultedValue::NoDefault
    }
}

impl<T> BaeDefault for FnCallVarArgs<T>
where
    T: BaeParse,
{
    fn bae_default() -> BaeDefaultedValue<Self> {
        BaeDefaultedValue::NoDefault
    }
}

impl<T> FnCallFixed<T>
where
    T: Sized,
{
    fn parse_with_inner_parser<P>(input: ParseStream, parser: &mut P) -> BaeParseResult<Self>
    where
        P: FnMut(ParseStream) -> Result<T>,
    {
        let content;
        syn::parenthesized!(content in input);

        let span = {
            let fork = content.fork();
            let ts: TokenStream = fork.parse()?;
            ts.span()
        };

        let inner = parser(&content)?;

        Ok(BaeSpanned::new(Self { inner }, Some(span)))
    }
}

impl<T> FnCallVarArgs<T> where T: Sized {}

impl<T> AsRef<T> for FnCallFixed<T>
where
    T: Sized,
{
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> AsRef<Vec<T>> for FnCallVarArgs<T>
where
    T: Sized,
{
    fn as_ref(&self) -> &Vec<T> {
        &self.inner
    }
}

impl<'a, T> Deref for FnCallFixed<T>
where
    Self: 'a,
    T: BaeParse,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<'a, T> Deref for FnCallVarArgs<T>
where
    Self: 'a,
    T: BaeParse,
{
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> BaeParse for FnCallVarArgs<T>
where
    T: BaeParse,
{
    fn parse(input: ParseStream) -> BaeParseResult<Self> {
        let content;
        syn::parenthesized!(content in input);

        let span = {
            let fork = content.fork();
            let ts: TokenStream = fork.parse()?;
            ts.span()
        };

        let mut inner = Vec::new();

        while !content.is_empty() {
            inner.push(<T as BaeParse>::parse_fn_arg(&content)?.unwrap());
            if !content.peek(syn::Token![,]) {
                break;
            }
            content.parse::<syn::Token![,]>()?;
        }

        if !content.is_empty() {
            return Err(Error::new(input.span(), "Invalid arguments"));
        }

        Ok(BaeSpanned::new(Self { inner }, Some(span)))
    }
    fn parse_prefix(input: ParseStream) -> BaeParseResult<Self> {
        Self::parse(input)
    }
}

impl<T> BaeSupportedOtherType for FnCallVarArgs<T> {}
impl<T> BaeSupportedAllType for FnCallVarArgs<T> {}
impl<T> BaeSupportedTypeBunked for FnCallVarArgs<T> where FnCallVarArgs<T>: BaeDefault {}

macro_rules! impl_bae_parse_fn_call_mixed {
    ($($x:ident),*) => {
        #[allow(unused_parens, non_snake_case)]
        impl< $($x),* > BaeParse for FnCallFixed<( $( $x, )* )>
        where
            $(
                $x: BaeParse,
            )*
        {
            fn parse(input: ParseStream) -> BaeParseResult<Self> {
                Self::parse_with_inner_parser(input, &mut |input| -> Result<( $( $x, )* )> {
                    paste! {
                        $(
                            let [< var_ $x >] = <$x as BaeParse>::parse_fn_arg(input)?.unwrap();
                            if !input.is_empty() {
                                input.parse::<syn::Token![,]>()?;
                            }
                        )*
                        if !input.is_empty() {
                            return Err(Error::new(input.span(), "Too many arguments"));
                        }

                        Ok(( $( [< var_ $x >], )* ))
                    }
                })
            }
            fn parse_prefix(input: ParseStream) -> BaeParseResult<Self> {
                Self::parse(input)
            }
        }

        impl< $($x),* > BaeSupportedOtherType for FnCallFixed<( $( $x, )* )> {}
        impl< $($x),* > BaeSupportedAllType for FnCallFixed<( $( $x, )* )> {}
        impl< $($x),* > BaeSupportedTypeBunked for FnCallFixed<( $( $x, )* )> {}
    };
}

macro_rules! impl_bae_parse_fn_call_mixed_multi {
    ($x:ident) => {
        impl_bae_parse_fn_call_mixed!($x);
    };
    ($x:ident, $($y:ident),*) => {
        impl_bae_parse_fn_call_mixed!($x, $($y),*);
        impl_bae_parse_fn_call_mixed_multi!($($y),*);
    };
}

impl_bae_parse_fn_call_mixed_multi!(L, K, J, I, H, G, F, E, D, C, B, A);
