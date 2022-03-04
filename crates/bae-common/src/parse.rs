use proc_macro2::{Ident, Span};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Result,
};

use syn::{Error, LitFloat, LitInt, LitStr};

/// Result of a `BaeParse::bae_parse*` call
pub type BaeParseResult<T> = Result<BaeSpanned<T>>;

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
    fn parse(input: ParseStream) -> BaeParseResult<Self>;

    /// Parse the `input` `ParseStream` with (by default) `=` prefix
    fn parse_prefix(input: ParseStream) -> BaeParseResult<Self> {
        input.parse::<syn::Token![=]>()?;
        <Self as BaeParse>::parse(input)
    }

    /// Parse the `input` `ParseStream` like a function argument (e.g. for `Option<u8>` take ident("None") to be None, and Some("123") to be Some(LitStr("123")))
    fn parse_fn_arg(input: ParseStream) -> BaeParseResult<Self> {
        <Self as BaeParse>::parse(input)
    }
}

/// Parsing interface implemented by types that
pub trait BaeParseVia
where
    Self: Sized,
{
    /// Parse 'via' this type
    type Via: Parse + Spanned;

    /// Try conversion from `Self::Via` to `Self`
    fn try_via(via: Self::Via) -> Result<Self>;
}

impl<T: BaeParseVia> BaeParse for T
where
    Self: BaeParseVia,
    <Self as BaeParseVia>::Via: BaeParse,
{
    fn parse(input: ParseStream) -> BaeParseResult<Self> {
        <<Self as BaeParseVia>::Via as BaeParse>::parse(input)?
            .map(<Self as BaeParseVia>::try_via)
            .transpose()
    }

    fn parse_prefix(input: ParseStream) -> BaeParseResult<Self> {
        <<Self as BaeParseVia>::Via as BaeParse>::parse_prefix(input)?
            .map(<Self as BaeParseVia>::try_via)
            .transpose()
    }

    fn parse_fn_arg(input: ParseStream) -> BaeParseResult<Self> {
        <<Self as BaeParseVia>::Via as BaeParse>::parse_fn_arg(input)?
            .map(<Self as BaeParseVia>::try_via)
            .transpose()
    }
}

impl BaeParse for () {
    fn parse(_input: ParseStream) -> BaeParseResult<Self> {
        Ok(BaeSpanned::new((), None))
    }

    fn parse_prefix(input: ParseStream) -> BaeParseResult<Self> {
        Self::parse(input)
    }
}

impl<T> BaeParse for Option<T>
where
    T: BaeParse,
{
    fn parse(input: ParseStream) -> BaeParseResult<Self> {
        Ok(<T as BaeParse>::parse(input)?.map(|v| Some(v)))
    }

    fn parse_prefix(input: ParseStream) -> BaeParseResult<Self> {
        Ok(<T as BaeParse>::parse_prefix(input)?.map(|v| Some(v)))
    }

    fn parse_fn_arg(input: ParseStream) -> BaeParseResult<Self> {
        let variant_name = input.parse::<Ident>()?;
        match variant_name.to_string().as_str() {
            "None" => Ok(BaeSpanned::new(None, Some(variant_name.span()))),
            "Some" => {
                let content;
                syn::parenthesized!(content in input);
                let inner = T::parse_fn_arg(&content)?.map(|v| Some(v));
                Ok(inner)
            }
            _ => Err(Error::new(input.span(), "Invalid Option variant")),
        }
    }
}

impl BaeParseVia for String {
    type Via = LitStr;

    fn try_via(via: Self::Via) -> Result<Self> {
        Ok(via.value())
    }
}

macro_rules! impl_bae_parse_syn_type {
    ($x:ty) => {
        impl BaeParse for $x
        where
            Self: Parse + BaeSupportedSynType,
        {
            fn parse(input: ParseStream) -> BaeParseResult<Self> {
                let inner = <Self as Parse>::parse(input)?;

                Ok(BaeSpanned::from(inner))
            }
        }
    };
}

pub(crate) use impl_bae_parse_syn_type;

macro_rules! impl_bae_parse_via_integer_types {
    ($($x:ty),+) => (
        $(
            impl BaeParseVia for $x {
                type Via = LitInt;

                fn try_via(via: Self::Via) -> Result<Self> {
                    via.base10_parse()
                }
            }
        )+
    );
}

impl_bae_parse_via_integer_types!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

macro_rules! impl_bae_parse_via_float_types {
    ($($x:ty),+) => (
        $(
            impl BaeParseVia for $x {
                type Via = LitFloat;

                fn try_via(via: Self::Via) -> Result<Self> {
                    via.base10_parse()
                }
            }
        )+
    );
}

impl_bae_parse_via_float_types!(f32, f64);

/// A "Spanned" value - the result of `BaeParse::bae_parse`
pub struct BaeSpanned<T> {
    inner: T,
    span: Option<Span>,
}

impl<T> BaeSpanned<T> {
    /// Create a new `BaeSpanned<T>` with optional `Span`
    pub fn new(inner: T, span: Option<Span>) -> Self {
        Self { inner, span }
    }

    /// Create a new `BaeSpanned<T>` with `Span` from `syn::spanned::Spanned` inner
    pub fn from(inner: T) -> Self
    where
        T: Spanned,
    {
        let span = Some(inner.span());

        Self { inner, span }
    }

    /// Unwrap the inner value
    pub fn unwrap(self) -> T {
        self.inner
    }

    /// Unwrap the inner value and the `Span`
    pub fn unwrap_with_span(self) -> (T, Option<Span>) {
        (self.inner, self.span)
    }

    /// Retrieve the `Span`
    pub fn span(&self) -> Option<Span> {
        self.span
    }

    /// Map this `BaeSpanned`, creating a new `BaeSpanned` with inner being the return value of the mapper function
    pub fn map<U, F>(self, f: F) -> BaeSpanned<U>
    where
        F: FnOnce(T) -> U,
    {
        let inner = f(self.inner);
        let span = self.span;

        BaeSpanned { inner, span }
    }

    /// Map this `BaeSpanned`, creating a new `BaeSpanned` with inner being the return value of the mapper function
    pub fn map_with_span<U, F>(self, f: F) -> BaeSpanned<U>
    where
        F: FnOnce(T, Option<Span>) -> U,
    {
        let span = self.span;
        let inner = f(self.inner, span);

        BaeSpanned { inner, span }
    }

    /// Convert this `BaeSpanned` into a `BaeSpanned` containing a reference to the original's value
    pub fn as_ref(&self) -> BaeSpanned<&T> {
        BaeSpanned {
            inner: &self.inner,
            span: self.span,
        }
    }
}

impl<T, E> BaeSpanned<std::result::Result<T, E>> {
    /// Convert `BaeSpanned<Result<T, E>>` into `Result<BaeSpanned<T>, E>`
    pub fn transpose(self) -> std::result::Result<BaeSpanned<T>, E> {
        match self.inner {
            Ok(inner) => Ok(BaeSpanned {
                inner,
                span: self.span,
            }),
            Err(e) => Err(e),
        }
    }
}
