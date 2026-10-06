#![feature(prelude_import)]
//! `#[derive(CgpVariant)]` on an enum with a variant that has no fields, in a
//! `no_std` crate that brings `Box` into scope. The mutable extractor builds an
//! empty variant's `&mut Nil` payload with `Box::leak(Box::new(Nil))`, writing
//! `Box` bare so it resolves where the derive is used, and here it resolves to
//! the imported `std::boxed::Box`. `extern crate std;` keeps the binary linkable
//! while `#![no_std]` drops the `std` prelude, so the import is what puts `Box`
//! in scope. The program compiles clean.
#![no_std]
extern crate core;
#[prelude_import]
use core::prelude::rust_2024::*;
extern crate std;
use std::boxed::Box;
use cgp::prelude::*;
pub enum Status {
    Active(u64),
    Closed,
}
impl HasFields for Status {
    type Fields = Enum! { Active(u64), Closed };
}
impl HasFieldsRef for Status {
    type FieldsRef<'__a> = Enum! { Active(&'__a u64), Closed }
    where
        Self: '__a;
}
impl FromFields for Status {
    fn from_fields(rest: Self::Fields) -> Self {
        match rest {
            Either::Left(field) => {
                let field = field.value;
                Self::Active(field)
            }
            Either::Right(rest) => {
                match rest {
                    Either::Left(field) => {
                        let Nil = field.value;
                        Self::Closed
                    }
                    Either::Right(rest) => match rest {}
                }
            }
        }
    }
}
impl ToFields for Status {
    fn to_fields(self) -> Self::Fields {
        match self {
            Self::Active(field) => Either::Left(field.into()),
            Self::Closed => Either::Right(Either::Left(Nil.into())),
        }
    }
}
impl ToFieldsRef for Status {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        match self {
            Self::Active(field) => Either::Left(field.into()),
            Self::Closed => Either::Right(Either::Left(Nil.into())),
        }
    }
}
impl FromVariant<Symbol!("Active")> for Status {
    type Value = u64;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Active")>,
        value: Self::Value,
    ) -> Self {
        Self::Active(value)
    }
}
impl FromVariant<Symbol!("Closed")> for Status {
    type Value = Nil;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Closed")>,
        _: Self::Value,
    ) -> Self {
        Self::Closed {}
    }
}
pub enum __PartialStatus<__F0__: MapType, __F1__: MapType> {
    Active(<__F0__ as MapType>::Map<u64>),
    Closed(<__F1__ as MapType>::Map<Nil>),
}
pub enum __PartialRefStatus<
    '__a__,
    __R__: MapTypeRef,
    __F0__: MapType,
    __F1__: MapType,
> {
    Active(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, u64>>),
    Closed(<__F1__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, Nil>>),
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialStatus<__F0__, __F1__> {
    type Target = Status;
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> PartialData
for __PartialRefStatus<'__a__, __R__, __F0__, __F1__> {
    type Target = Status;
}
impl HasExtractor for Status {
    type Extractor = __PartialStatus<IsPresent, IsPresent>;
    fn to_extractor(self) -> Self::Extractor {
        match self {
            Self::Active(value) => __PartialStatus::Active(value),
            Self::Closed { .. } => __PartialStatus::Closed(Nil),
        }
    }
    fn from_extractor(extractor: Self::Extractor) -> Self {
        match extractor {
            __PartialStatus::Active(value) => Self::Active(value),
            __PartialStatus::Closed(_) => Self::Closed {},
        }
    }
}
impl HasExtractorRef for Status {
    type ExtractorRef<'__a__> = __PartialRefStatus<'__a__, IsRef, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_ref<'__a__>(&'__a__ self) -> Self::ExtractorRef<'__a__> {
        match self {
            Self::Active(value) => __PartialRefStatus::Active(value),
            Self::Closed { .. } => __PartialRefStatus::Closed(&Nil),
        }
    }
}
impl HasExtractorMut for Status {
    type ExtractorMut<'__a__> = __PartialRefStatus<'__a__, IsMut, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_mut<'__a__>(&'__a__ mut self) -> Self::ExtractorMut<'__a__> {
        match self {
            Self::Active(value) => __PartialRefStatus::Active(value),
            Self::Closed { .. } => __PartialRefStatus::Closed(Box::leak(Box::new(Nil))),
        }
    }
}
impl FinalizeExtract for __PartialStatus<IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<'__a__, __R__: MapTypeRef> FinalizeExtract
for __PartialRefStatus<'__a__, __R__, IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<__F1__: MapType> ExtractField<Symbol!("Active")>
for __PartialStatus<IsPresent, __F1__> {
    type Value = u64;
    type Remainder = __PartialStatus<IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Active")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialStatus::Active(value) => Ok(value),
            __PartialStatus::Closed(value) => Err(__PartialStatus::Closed(value)),
        }
    }
}
impl<__F0__: MapType> ExtractField<Symbol!("Closed")>
for __PartialStatus<__F0__, IsPresent> {
    type Value = Nil;
    type Remainder = __PartialStatus<__F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Closed")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialStatus::Active(value) => Err(__PartialStatus::Active(value)),
            __PartialStatus::Closed(value) => Ok(value),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F1__: MapType> ExtractField<Symbol!("Active")>
for __PartialRefStatus<'__a__, __R__, IsPresent, __F1__> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, u64>;
    type Remainder = __PartialRefStatus<'__a__, __R__, IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Active")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialRefStatus::Active(value) => Ok(value),
            __PartialRefStatus::Closed(value) => Err(__PartialRefStatus::Closed(value)),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType> ExtractField<Symbol!("Closed")>
for __PartialRefStatus<'__a__, __R__, __F0__, IsPresent> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, Nil>;
    type Remainder = __PartialRefStatus<'__a__, __R__, __F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Closed")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialRefStatus::Active(value) => Err(__PartialRefStatus::Active(value)),
            __PartialRefStatus::Closed(value) => Ok(value),
        }
    }
}
fn main() {
    let mut status = Status::Closed;
    let _ = status.extractor_mut();
    let _ = Status::Active(1);
}
