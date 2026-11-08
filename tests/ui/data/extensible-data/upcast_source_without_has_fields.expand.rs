#![feature(prelude_import)]
//! Upcasting from an enum that derives only `ExtractField`.
//!
//! An upcast walks the source's `HasFields` list, so the source needs `#[derive(HasFields)]` too.
//! rustc reports "no method named `upcast` found for enum `Small`"; the tool names the missing
//! trait and its derive: ``[CGP-E050] `Small` does not implement `HasFields` ``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::field::impls::CanUpcast;
use cgp::prelude::*;
pub enum Small {
    Foo(u64),
}
///The partial extractor for [`Small`].
pub enum __PartialSmall<__F0__: MapType> {
    Foo(<__F0__ as MapType>::Map<u64>),
}
///The borrowed partial extractor for [`Small`].
pub enum __PartialRefSmall<'__a__, __R__: MapTypeRef, __F0__: MapType> {
    Foo(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, u64>>),
}
impl<__F0__: MapType> PartialData for __PartialSmall<__F0__> {
    type Target = Small;
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType> PartialData
for __PartialRefSmall<'__a__, __R__, __F0__> {
    type Target = Small;
}
impl HasExtractor for Small {
    type Extractor = __PartialSmall<IsPresent>;
    fn to_extractor(self) -> <Self as HasExtractor>::Extractor {
        match self {
            Self::Foo(value) => __PartialSmall::Foo(value),
        }
    }
    fn from_extractor(extractor: <Self as HasExtractor>::Extractor) -> Self {
        match extractor {
            __PartialSmall::Foo(value) => Self::Foo(value),
        }
    }
}
impl HasExtractorRef for Small {
    type ExtractorRef<'__a__> = __PartialRefSmall<'__a__, IsRef, IsPresent>
    where
        Self: '__a__;
    fn extractor_ref<'__a__>(
        &'__a__ self,
    ) -> <Self as HasExtractorRef>::ExtractorRef<'__a__> {
        match self {
            Self::Foo(value) => __PartialRefSmall::Foo(value),
        }
    }
}
impl HasExtractorMut for Small {
    type ExtractorMut<'__a__> = __PartialRefSmall<'__a__, IsMut, IsPresent>
    where
        Self: '__a__;
    fn extractor_mut<'__a__>(
        &'__a__ mut self,
    ) -> <Self as HasExtractorMut>::ExtractorMut<'__a__> {
        match self {
            Self::Foo(value) => __PartialRefSmall::Foo(value),
        }
    }
}
impl FinalizeExtract for __PartialSmall<IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<'__a__, __R__: MapTypeRef> FinalizeExtract
for __PartialRefSmall<'__a__, __R__, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl ExtractField<Symbol!("Foo")> for __PartialSmall<IsPresent> {
    type Value = u64;
    type Remainder = __PartialSmall<IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Foo")>>::Value,
        <Self as ExtractField<Symbol!("Foo")>>::Remainder,
    > {
        match self {
            __PartialSmall::Foo(value) => Ok(value),
        }
    }
}
impl<'__a__, __R__: MapTypeRef> ExtractField<Symbol!("Foo")>
for __PartialRefSmall<'__a__, __R__, IsPresent> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, u64>;
    type Remainder = __PartialRefSmall<'__a__, __R__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Foo")>>::Value,
        <Self as ExtractField<Symbol!("Foo")>>::Remainder,
    > {
        match self {
            __PartialRefSmall::Foo(value) => Ok(value),
        }
    }
}
pub enum Big {
    Foo(u64),
    Bar(String),
}
impl HasFields for Big {
    type Fields = Enum! { Foo(u64), Bar(String) };
}
impl HasFieldsRef for Big {
    type FieldsRef<'__a> = Enum! { Foo(&'__a u64), Bar(&'__a String) }
    where
        Self: '__a;
}
impl FromFields for Big {
    fn from_fields(rest: <Self as HasFields>::Fields) -> Self {
        match rest {
            Either::Left(field) => {
                let field = field.value;
                Self::Foo(field)
            }
            Either::Right(rest) => {
                match rest {
                    Either::Left(field) => {
                        let field = field.value;
                        Self::Bar(field)
                    }
                    Either::Right(rest) => match rest {}
                }
            }
        }
    }
}
impl ToFields for Big {
    fn to_fields(self) -> <Self as HasFields>::Fields {
        match self {
            Self::Foo(field) => Either::Left(::core::convert::Into::into(field)),
            Self::Bar(field) => {
                Either::Right(Either::Left(::core::convert::Into::into(field)))
            }
        }
    }
}
impl ToFieldsRef for Big {
    fn to_fields_ref<'__a>(&'__a self) -> <Self as HasFieldsRef>::FieldsRef<'__a>
    where
        Self: '__a,
    {
        match self {
            Self::Foo(field) => Either::Left(::core::convert::Into::into(field)),
            Self::Bar(field) => {
                Either::Right(Either::Left(::core::convert::Into::into(field)))
            }
        }
    }
}
impl FromVariant<Symbol!("Foo")> for Big {
    type Value = u64;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
        value: <Self as FromVariant<Symbol!("Foo")>>::Value,
    ) -> Self {
        Self::Foo(value)
    }
}
impl FromVariant<Symbol!("Bar")> for Big {
    type Value = String;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Bar")>,
        value: <Self as FromVariant<Symbol!("Bar")>>::Value,
    ) -> Self {
        Self::Bar(value)
    }
}
///The partial extractor for [`Big`].
pub enum __PartialBig<__F0__: MapType, __F1__: MapType> {
    Foo(<__F0__ as MapType>::Map<u64>),
    Bar(<__F1__ as MapType>::Map<String>),
}
///The borrowed partial extractor for [`Big`].
pub enum __PartialRefBig<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> {
    Foo(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, u64>>),
    Bar(<__F1__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, String>>),
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialBig<__F0__, __F1__> {
    type Target = Big;
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> PartialData
for __PartialRefBig<'__a__, __R__, __F0__, __F1__> {
    type Target = Big;
}
impl HasExtractor for Big {
    type Extractor = __PartialBig<IsPresent, IsPresent>;
    fn to_extractor(self) -> <Self as HasExtractor>::Extractor {
        match self {
            Self::Foo(value) => __PartialBig::Foo(value),
            Self::Bar(value) => __PartialBig::Bar(value),
        }
    }
    fn from_extractor(extractor: <Self as HasExtractor>::Extractor) -> Self {
        match extractor {
            __PartialBig::Foo(value) => Self::Foo(value),
            __PartialBig::Bar(value) => Self::Bar(value),
        }
    }
}
impl HasExtractorRef for Big {
    type ExtractorRef<'__a__> = __PartialRefBig<'__a__, IsRef, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_ref<'__a__>(
        &'__a__ self,
    ) -> <Self as HasExtractorRef>::ExtractorRef<'__a__> {
        match self {
            Self::Foo(value) => __PartialRefBig::Foo(value),
            Self::Bar(value) => __PartialRefBig::Bar(value),
        }
    }
}
impl HasExtractorMut for Big {
    type ExtractorMut<'__a__> = __PartialRefBig<'__a__, IsMut, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_mut<'__a__>(
        &'__a__ mut self,
    ) -> <Self as HasExtractorMut>::ExtractorMut<'__a__> {
        match self {
            Self::Foo(value) => __PartialRefBig::Foo(value),
            Self::Bar(value) => __PartialRefBig::Bar(value),
        }
    }
}
impl FinalizeExtract for __PartialBig<IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<'__a__, __R__: MapTypeRef> FinalizeExtract
for __PartialRefBig<'__a__, __R__, IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<__F1__: MapType> ExtractField<Symbol!("Foo")> for __PartialBig<IsPresent, __F1__> {
    type Value = u64;
    type Remainder = __PartialBig<IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Foo")>>::Value,
        <Self as ExtractField<Symbol!("Foo")>>::Remainder,
    > {
        match self {
            __PartialBig::Foo(value) => Ok(value),
            __PartialBig::Bar(value) => Err(__PartialBig::Bar(value)),
        }
    }
}
impl<__F0__: MapType> ExtractField<Symbol!("Bar")> for __PartialBig<__F0__, IsPresent> {
    type Value = String;
    type Remainder = __PartialBig<__F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Bar")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Bar")>>::Value,
        <Self as ExtractField<Symbol!("Bar")>>::Remainder,
    > {
        match self {
            __PartialBig::Foo(value) => Err(__PartialBig::Foo(value)),
            __PartialBig::Bar(value) => Ok(value),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F1__: MapType> ExtractField<Symbol!("Foo")>
for __PartialRefBig<'__a__, __R__, IsPresent, __F1__> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, u64>;
    type Remainder = __PartialRefBig<'__a__, __R__, IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Foo")>>::Value,
        <Self as ExtractField<Symbol!("Foo")>>::Remainder,
    > {
        match self {
            __PartialRefBig::Foo(value) => Ok(value),
            __PartialRefBig::Bar(value) => Err(__PartialRefBig::Bar(value)),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType> ExtractField<Symbol!("Bar")>
for __PartialRefBig<'__a__, __R__, __F0__, IsPresent> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, String>;
    type Remainder = __PartialRefBig<'__a__, __R__, __F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Bar")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Bar")>>::Value,
        <Self as ExtractField<Symbol!("Bar")>>::Remainder,
    > {
        match self {
            __PartialRefBig::Foo(value) => Err(__PartialRefBig::Foo(value)),
            __PartialRefBig::Bar(value) => Ok(value),
        }
    }
}
fn main() {
    let _ = Small::Foo(1).upcast(PhantomData::<Big>);
}
