//! Types supported by `bae`'s `BaeParser` trait

use syn::parse::Parse;

use syn::{
    BinOp, Expr, ExprArray, ExprAssign, ExprAssignOp, ExprBinary, ExprCall, ExprCast, ExprClosure,
    ExprField, ExprIndex, ExprLit, ExprParen, ExprPath, ExprRange, ExprReference, ExprTry,
    ExprTuple, ExprType, ExprUnary, GenericArgument, Ident, Index, Lifetime, Lit, LitBool, LitByte,
    LitByteStr, LitChar, LitFloat, LitInt, LitStr, Meta, MetaList, MetaNameValue, NestedMeta, Path,
    Type, TypeArray, TypeBareFn, TypeGroup, TypeImplTrait, TypeInfer, TypeMacro, TypeNever,
    TypeParam, TypeParen, TypePath, TypePtr, TypeReference, TypeSlice, TypeTraitObject, TypeTuple,
    UnOp, Visibility,
};

use crate::{BaeDefault, BaeParse};

/// Implemented for `syn` types `bae` supports parsing
pub trait BaeSupportedSynType {}
/// Implemented for other types `bae` supports parsing
pub trait BaeSupportedOtherType {}
/// Implemented for all types `bae` supports parsing
pub trait BaeSupportedAllType {}
/// Implemented for all types that should support `BaeParse` and `BaeDefault`
pub trait BaeSupportedTypeChecked: BaeParse + BaeDefault {}

macro_rules! impl_bae_supported_syn_types {
    ($($x:ty),+) => (
        $(
            impl BaeSupportedSynType for $x
            where
                Self: Parse,
            {}
            impl BaeSupportedAllType for $x
            where
                Self: Parse,
            {}
            impl BaeSupportedTypeChecked for $x {}
        )+
    );
}

macro_rules! impl_bae_supported_other_types {
    ($($x:ty),+) => (
        $(
            impl BaeSupportedOtherType for $x {}
            impl BaeSupportedAllType for $x {}
            impl BaeSupportedTypeChecked for $x {}
        )+
    );
}

impl_bae_supported_syn_types!(
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

impl_bae_supported_other_types!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);

pub(crate) use impl_bae_supported_other_types;
