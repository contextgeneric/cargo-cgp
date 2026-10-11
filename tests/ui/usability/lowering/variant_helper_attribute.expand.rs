#![feature(prelude_import)]
//! A variant helper attribute of another derive on a `#[derive(CgpData)]` enum.
//!
//! `#[default]` belongs to `#[derive(Default)]`, which the enum has, but the extractor's companion
//! enums copy the variant attribute without the derive. The output says the attribute cannot be found
//! and twice suggests adding the `#[derive(Default)]` that is already there.
//!
//! CGP error class: none; a `cgp` derive defect recorded in the extractor's Known issues.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub enum Mode {
    #[default]
    Off,
    On(u64),
}
#[automatically_derived]
impl ::core::default::Default for Mode {
    #[inline]
    fn default() -> Self {
        Self::Off
    }
}
impl HasFields for Mode {
    type Fields = Enum! { Off, On(u64) };
}
impl HasFieldsRef for Mode {
    type FieldsRef<'__a> = Enum! { Off, On(&'__a u64) }
    where
        Self: '__a;
}
impl FromFields for Mode {
    fn from_fields(rest: Self::Fields) -> Self {
        match rest {
            Either::Left(field) => {
                let Nil = field.value;
                Self::Off
            }
            Either::Right(rest) => {
                match rest {
                    Either::Left(field) => {
                        let field = field.value;
                        Self::On(field)
                    }
                    Either::Right(rest) => match rest {}
                }
            }
        }
    }
}
impl ToFields for Mode {
    fn to_fields(self) -> Self::Fields {
        match self {
            Self::Off => Either::Left(Nil.into()),
            Self::On(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl ToFieldsRef for Mode {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        match self {
            Self::Off => Either::Left(Nil.into()),
            Self::On(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl FromVariant<Symbol!("Off")> for Mode {
    type Value = Nil;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Off")>,
        _: Self::Value,
    ) -> Self {
        Self::Off {}
    }
}
impl FromVariant<Symbol!("On")> for Mode {
    type Value = u64;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("On")>,
        value: Self::Value,
    ) -> Self {
        Self::On(value)
    }
}
pub enum __PartialMode<__F0__: MapType, __F1__: MapType> {
    #[default]
    Off(<__F0__ as MapType>::Map<Nil>),
    On(<__F1__ as MapType>::Map<u64>),
}
pub enum __PartialRefMode<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> {
    #[default]
    Off(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, Nil>>),
    On(<__F1__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, u64>>),
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialMode<__F0__, __F1__> {
    type Target = Mode;
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> PartialData
for __PartialRefMode<'__a__, __R__, __F0__, __F1__> {
    type Target = Mode;
}
impl HasExtractor for Mode {
    type Extractor = __PartialMode<IsPresent, IsPresent>;
    fn to_extractor(self) -> Self::Extractor {
        match self {
            Self::Off { .. } => __PartialMode::Off(Nil),
            Self::On(value) => __PartialMode::On(value),
        }
    }
    fn from_extractor(extractor: Self::Extractor) -> Self {
        match extractor {
            __PartialMode::Off(_) => Self::Off {},
            __PartialMode::On(value) => Self::On(value),
        }
    }
}
impl HasExtractorRef for Mode {
    type ExtractorRef<'__a__> = __PartialRefMode<'__a__, IsRef, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_ref<'__a__>(&'__a__ self) -> Self::ExtractorRef<'__a__> {
        match self {
            Self::Off { .. } => __PartialRefMode::Off(&Nil),
            Self::On(value) => __PartialRefMode::On(value),
        }
    }
}
impl HasExtractorMut for Mode {
    type ExtractorMut<'__a__> = __PartialRefMode<'__a__, IsMut, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_mut<'__a__>(&'__a__ mut self) -> Self::ExtractorMut<'__a__> {
        match self {
            Self::Off { .. } => __PartialRefMode::Off(Box::leak(Box::new(Nil))),
            Self::On(value) => __PartialRefMode::On(value),
        }
    }
}
impl FinalizeExtract for __PartialMode<IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<'__a__, __R__: MapTypeRef> FinalizeExtract
for __PartialRefMode<'__a__, __R__, IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<__F1__: MapType> ExtractField<Symbol!("Off")> for __PartialMode<IsPresent, __F1__> {
    type Value = Nil;
    type Remainder = __PartialMode<IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Off")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialMode::Off(value) => Ok(value),
            __PartialMode::On(value) => Err(__PartialMode::On(value)),
        }
    }
}
impl<__F0__: MapType> ExtractField<Symbol!("On")> for __PartialMode<__F0__, IsPresent> {
    type Value = u64;
    type Remainder = __PartialMode<__F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("On")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialMode::Off(value) => Err(__PartialMode::Off(value)),
            __PartialMode::On(value) => Ok(value),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F1__: MapType> ExtractField<Symbol!("Off")>
for __PartialRefMode<'__a__, __R__, IsPresent, __F1__> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, Nil>;
    type Remainder = __PartialRefMode<'__a__, __R__, IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Off")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialRefMode::Off(value) => Ok(value),
            __PartialRefMode::On(value) => Err(__PartialRefMode::On(value)),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType> ExtractField<Symbol!("On")>
for __PartialRefMode<'__a__, __R__, __F0__, IsPresent> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, u64>;
    type Remainder = __PartialRefMode<'__a__, __R__, __F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("On")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialRefMode::Off(value) => Err(__PartialRefMode::Off(value)),
            __PartialRefMode::On(value) => Ok(value),
        }
    }
}
fn main() {}
