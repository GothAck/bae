use syn::{
    parse::{Parse, ParseStream},
    Result,
};

use syn::{
    BinOp, Expr, ExprArray, ExprAssign, ExprAssignOp, ExprBinary, ExprCall, ExprCast, ExprClosure,
    ExprField, ExprIndex, ExprLit, ExprParen, ExprPath, ExprRange, ExprReference, ExprTry,
    ExprTuple, ExprType, ExprUnary, GenericArgument, Ident, Index, Lifetime, Lit, LitBool, LitByte,
    LitByteStr, LitChar, LitFloat, LitInt, LitStr, Meta, MetaList, MetaNameValue, NestedMeta, Path,
    Type, TypeArray, TypeBareFn, TypeGroup, TypeImplTrait, TypeInfer, TypeMacro, TypeNever,
    TypeParam, TypeParen, TypePath, TypePtr, TypeReference, TypeSlice, TypeTraitObject, TypeTuple,
    UnOp, Visibility,
};

/// Parsing interface implemented by all types that can be parsed in a default way by a `FromAttribute` implementation.
///
/// Equivalent to `syn::parse::Parse`, however this is not implemented for every type that implements `syn::parse::Parse` automatically.
/// This is due to our special case parsing of:
/// `()`
///     Used in `Option<()>` switches
/// `Option<T> where T: BaeParse`
///     Used in `Option<()>` switches
/// `String`
///     Parses via `syn::LitStr`
/// `u8`, `u16`, `u32`, `u64`, `u128`, `usize`, `i8`, `i16`, `i32`, `i64`, `i128`, `isize`
///     Parses via `syn::LitInt`
/// `f32`, `f64`
///     Parses via `syn::LitFloat`
pub trait BaeParse
where
    Self: Sized,
{
    /// Parse the `input` ParseStream
    fn parse(input: ParseStream) -> Result<Self>;

    /// Parse the `input` `ParseStream` with (by default) `=` prefix
    fn parse_prefix(input: ParseStream) -> Result<Self> {
        input.parse::<syn::Token![=]>()?;
        <Self as BaeParse>::parse(input)
    }
}

impl BaeParse for () {
    fn parse(_input: ParseStream) -> Result<Self> {
        Ok(())
    }

    fn parse_prefix(_input: ParseStream) -> Result<Self> {
        Ok(())
    }
}

impl<T> BaeParse for Option<T>
where
    T: BaeParse,
{
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Some(<T as BaeParse>::parse(input)?))
    }

    fn parse_prefix(input: ParseStream) -> Result<Self> {
        Ok(Some(<T as BaeParse>::parse_prefix(input)?))
    }
}

impl BaeParse for String {
    fn parse(input: ParseStream) -> Result<Self> {
        let lit_str = <LitStr as BaeParse>::parse(input)?;
        Ok(lit_str.value())
    }
}

macro_rules! impl_bae_parse_syn_types {
    ($($x:ty),+) => (
        $(
            impl BaeParse for $x {
                fn parse(input: ParseStream) -> Result<Self> {

                    <Self as Parse>::parse(&input)
                }
            }
        )+
    );
}

impl_bae_parse_syn_types!(
    Expr,
    ExprArray,
    ExprAssign,
    ExprAssignOp,
    ExprBinary,
    ExprCall,
    ExprCast,
    ExprClosure,
    ExprField,
    ExprIndex,
    ExprLit,
    ExprParen,
    ExprPath,
    ExprRange,
    ExprReference,
    ExprTry,
    ExprTuple,
    ExprType,
    ExprUnary,
    Ident,
    Lit,
    LitBool,
    LitByte,
    LitByteStr,
    LitChar,
    LitFloat,
    LitInt,
    LitStr,
    Index,
    Lifetime,
    Path,
    Type,
    TypeArray,
    TypeBareFn,
    // TypeGenerics // Disabled as it has a lifetime specifier
    TypeGroup,
    TypeImplTrait,
    TypeInfer,
    TypeMacro,
    TypeNever,
    TypeParam,
    TypeParen,
    TypePath,
    TypePtr,
    TypeReference,
    TypeSlice,
    TypeTraitObject,
    TypeTuple,
    Visibility,
    BinOp,
    GenericArgument,
    Meta,
    MetaList,
    MetaNameValue,
    NestedMeta,
    UnOp
);

macro_rules! impl_bae_parse_integer_types {
    ($($x:ty),+) => (
        $(
            impl BaeParse for $x {
                fn parse(input: ParseStream) -> Result<Self> {
                    let lit_int = <LitInt as BaeParse>::parse(input)?;
                    lit_int.base10_parse()
                }
            }
        )+
    );
}

impl_bae_parse_integer_types!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

macro_rules! impl_bae_parse_float_types {
    ($($x:ty),+) => (
        $(
            impl BaeParse for $x {
                fn parse(input: ParseStream) -> Result<Self> {
                    let lit_int = <LitFloat as BaeParse>::parse(input)?;
                    lit_int.base10_parse()
                }
            }
        )+
    );
}

impl_bae_parse_float_types!(f32, f64);
