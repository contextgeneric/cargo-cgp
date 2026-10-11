#![feature(prelude_import)]
//! An enum variant named `Remainder` under `#[derive(CgpData)]`.
//!
//! The extractor slice's `Self::Remainder` collides with the variant, and as for `ExtractField` both
//! notes land on the derive attribute.
//!
//! CGP error class: out-of-scope generated name, its reserved-variant-name sibling
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/out-of-scope-generated-name.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub enum Token {
    Remainder(u64),
    Text(String),
}
impl HasFields for Token {
    type Fields = Enum! { Remainder(u64), Text(String) };
}
impl HasFieldsRef for Token {
    type FieldsRef<'__a> = Enum! { Remainder(&'__a u64), Text(&'__a String) }
    where
        Self: '__a;
}
impl FromFields for Token {
    fn from_fields(rest: Self::Fields) -> Self {
        match rest {
            Either::Left(field) => {
                let field = field.value;
                Self::Remainder(field)
            }
            Either::Right(rest) => {
                match rest {
                    Either::Left(field) => {
                        let field = field.value;
                        Self::Text(field)
                    }
                    Either::Right(rest) => match rest {}
                }
            }
        }
    }
}
impl ToFields for Token {
    fn to_fields(self) -> Self::Fields {
        match self {
            Self::Remainder(field) => Either::Left(field.into()),
            Self::Text(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl ToFieldsRef for Token {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        match self {
            Self::Remainder(field) => Either::Left(field.into()),
            Self::Text(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl FromVariant<Symbol!("Remainder")> for Token {
    type Value = u64;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Remainder")>,
        value: Self::Value,
    ) -> Self {
        Self::Remainder(value)
    }
}
impl FromVariant<Symbol!("Text")> for Token {
    type Value = String;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Text")>,
        value: Self::Value,
    ) -> Self {
        Self::Text(value)
    }
}
pub enum __PartialToken<__F0__: MapType, __F1__: MapType> {
    Remainder(<__F0__ as MapType>::Map<u64>),
    Text(<__F1__ as MapType>::Map<String>),
}
pub enum __PartialRefToken<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> {
    Remainder(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, u64>>),
    Text(<__F1__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, String>>),
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialToken<__F0__, __F1__> {
    type Target = Token;
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> PartialData
for __PartialRefToken<'__a__, __R__, __F0__, __F1__> {
    type Target = Token;
}
impl HasExtractor for Token {
    type Extractor = __PartialToken<IsPresent, IsPresent>;
    fn to_extractor(self) -> Self::Extractor {
        match self {
            Self::Remainder(value) => __PartialToken::Remainder(value),
            Self::Text(value) => __PartialToken::Text(value),
        }
    }
    fn from_extractor(extractor: Self::Extractor) -> Self {
        match extractor {
            __PartialToken::Remainder(value) => Self::Remainder(value),
            __PartialToken::Text(value) => Self::Text(value),
        }
    }
}
impl HasExtractorRef for Token {
    type ExtractorRef<'__a__> = __PartialRefToken<'__a__, IsRef, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_ref<'__a__>(&'__a__ self) -> Self::ExtractorRef<'__a__> {
        match self {
            Self::Remainder(value) => __PartialRefToken::Remainder(value),
            Self::Text(value) => __PartialRefToken::Text(value),
        }
    }
}
impl HasExtractorMut for Token {
    type ExtractorMut<'__a__> = __PartialRefToken<'__a__, IsMut, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_mut<'__a__>(&'__a__ mut self) -> Self::ExtractorMut<'__a__> {
        match self {
            Self::Remainder(value) => __PartialRefToken::Remainder(value),
            Self::Text(value) => __PartialRefToken::Text(value),
        }
    }
}
impl FinalizeExtract for __PartialToken<IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<'__a__, __R__: MapTypeRef> FinalizeExtract
for __PartialRefToken<'__a__, __R__, IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<__F1__: MapType> ExtractField<Symbol!("Remainder")>
for __PartialToken<IsPresent, __F1__> {
    type Value = u64;
    type Remainder = __PartialToken<IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Remainder")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialToken::Remainder(value) => Ok(value),
            __PartialToken::Text(value) => Err(__PartialToken::Text(value)),
        }
    }
}
impl<__F0__: MapType> ExtractField<Symbol!("Text")>
for __PartialToken<__F0__, IsPresent> {
    type Value = String;
    type Remainder = __PartialToken<__F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Text")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialToken::Remainder(value) => Err(__PartialToken::Remainder(value)),
            __PartialToken::Text(value) => Ok(value),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F1__: MapType> ExtractField<Symbol!("Remainder")>
for __PartialRefToken<'__a__, __R__, IsPresent, __F1__> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, u64>;
    type Remainder = __PartialRefToken<'__a__, __R__, IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Remainder")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialRefToken::Remainder(value) => Ok(value),
            __PartialRefToken::Text(value) => Err(__PartialRefToken::Text(value)),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType> ExtractField<Symbol!("Text")>
for __PartialRefToken<'__a__, __R__, __F0__, IsPresent> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, String>;
    type Remainder = __PartialRefToken<'__a__, __R__, __F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Text")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialRefToken::Remainder(value) => {
                Err(__PartialRefToken::Remainder(value))
            }
            __PartialRefToken::Text(value) => Ok(value),
        }
    }
}
fn main() {}
