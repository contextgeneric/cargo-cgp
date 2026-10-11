#![feature(prelude_import)]
//! An enum variant named `Value` under `#[derive(ExtractField)]`.
//!
//! The generated `Self::Value` is ambiguous with the variant. Both notes land on the derive
//! attribute, so the output never names the variant to rename. This is the opaque case.
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
    Value(u64),
    Text(String),
}
pub enum __PartialToken<__F0__: MapType, __F1__: MapType> {
    Value(<__F0__ as MapType>::Map<u64>),
    Text(<__F1__ as MapType>::Map<String>),
}
pub enum __PartialRefToken<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> {
    Value(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, u64>>),
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
            Self::Value(value) => __PartialToken::Value(value),
            Self::Text(value) => __PartialToken::Text(value),
        }
    }
    fn from_extractor(extractor: Self::Extractor) -> Self {
        match extractor {
            __PartialToken::Value(value) => Self::Value(value),
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
            Self::Value(value) => __PartialRefToken::Value(value),
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
            Self::Value(value) => __PartialRefToken::Value(value),
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
impl<__F1__: MapType> ExtractField<Symbol!("Value")>
for __PartialToken<IsPresent, __F1__> {
    type Value = u64;
    type Remainder = __PartialToken<IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Value")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialToken::Value(value) => Ok(value),
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
            __PartialToken::Value(value) => Err(__PartialToken::Value(value)),
            __PartialToken::Text(value) => Ok(value),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F1__: MapType> ExtractField<Symbol!("Value")>
for __PartialRefToken<'__a__, __R__, IsPresent, __F1__> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, u64>;
    type Remainder = __PartialRefToken<'__a__, __R__, IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Value")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialRefToken::Value(value) => Ok(value),
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
            __PartialRefToken::Value(value) => Err(__PartialRefToken::Value(value)),
            __PartialRefToken::Text(value) => Ok(value),
        }
    }
}
fn main() {}
