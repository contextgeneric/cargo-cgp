#![feature(prelude_import)]
//! Upcasting into a target whose same-named variant carries another payload type.
//!
//! `FooBar::Foo(u64)` into `FooBarBaz::Foo(u32)` is an `E0271` on
//! `<FooBarBaz as FromVariant<Symbol!("Foo")>>::Value == u64`. The tool reads the payload types:
//! ``[CGP-E050] variant `Foo` on `FooBarBaz` has type `u32`, but `u64` is required``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::field::impls::CanUpcast;
use cgp::prelude::*;
pub enum FooBar {
    Foo(u64),
    Bar(String),
}
#[automatically_derived]
impl ::core::fmt::Debug for FooBar {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Self::Foo(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Foo", &__self_0)
            }
            Self::Bar(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Bar", &__self_0)
            }
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for FooBar {}
#[automatically_derived]
impl ::core::cmp::PartialEq for FooBar {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        ::core::intrinsics::discriminant_value(self)
            == ::core::intrinsics::discriminant_value(other)
            && match (self, other) {
                (Self::Foo(__self_0), Self::Foo(__arg1_0)) => __self_0 == __arg1_0,
                (Self::Bar(__self_0), Self::Bar(__arg1_0)) => __self_0 == __arg1_0,
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
impl HasFields for FooBar {
    type Fields = Enum! { Foo(u64), Bar(String) };
}
impl HasFieldsRef for FooBar {
    type FieldsRef<'__a> = Enum! { Foo(&'__a u64), Bar(&'__a String) }
    where
        Self: '__a;
}
impl FromFields for FooBar {
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
impl ToFields for FooBar {
    fn to_fields(self) -> <Self as HasFields>::Fields {
        match self {
            Self::Foo(field) => Either::Left(field.into()),
            Self::Bar(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl ToFieldsRef for FooBar {
    fn to_fields_ref<'__a>(&'__a self) -> <Self as HasFieldsRef>::FieldsRef<'__a>
    where
        Self: '__a,
    {
        match self {
            Self::Foo(field) => Either::Left(field.into()),
            Self::Bar(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl FromVariant<Symbol!("Foo")> for FooBar {
    type Value = u64;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
        value: <Self as FromVariant<Symbol!("Foo")>>::Value,
    ) -> Self {
        Self::Foo(value)
    }
}
impl FromVariant<Symbol!("Bar")> for FooBar {
    type Value = String;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Bar")>,
        value: <Self as FromVariant<Symbol!("Bar")>>::Value,
    ) -> Self {
        Self::Bar(value)
    }
}
pub enum __PartialFooBar<__F0__: MapType, __F1__: MapType> {
    Foo(<__F0__ as MapType>::Map<u64>),
    Bar(<__F1__ as MapType>::Map<String>),
}
pub enum __PartialRefFooBar<
    '__a__,
    __R__: MapTypeRef,
    __F0__: MapType,
    __F1__: MapType,
> {
    Foo(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, u64>>),
    Bar(<__F1__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, String>>),
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialFooBar<__F0__, __F1__> {
    type Target = FooBar;
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> PartialData
for __PartialRefFooBar<'__a__, __R__, __F0__, __F1__> {
    type Target = FooBar;
}
impl HasExtractor for FooBar {
    type Extractor = __PartialFooBar<IsPresent, IsPresent>;
    fn to_extractor(self) -> <Self as HasExtractor>::Extractor {
        match self {
            Self::Foo(value) => __PartialFooBar::Foo(value),
            Self::Bar(value) => __PartialFooBar::Bar(value),
        }
    }
    fn from_extractor(extractor: <Self as HasExtractor>::Extractor) -> Self {
        match extractor {
            __PartialFooBar::Foo(value) => Self::Foo(value),
            __PartialFooBar::Bar(value) => Self::Bar(value),
        }
    }
}
impl HasExtractorRef for FooBar {
    type ExtractorRef<'__a__> = __PartialRefFooBar<'__a__, IsRef, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_ref<'__a__>(
        &'__a__ self,
    ) -> <Self as HasExtractorRef>::ExtractorRef<'__a__> {
        match self {
            Self::Foo(value) => __PartialRefFooBar::Foo(value),
            Self::Bar(value) => __PartialRefFooBar::Bar(value),
        }
    }
}
impl HasExtractorMut for FooBar {
    type ExtractorMut<'__a__> = __PartialRefFooBar<'__a__, IsMut, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_mut<'__a__>(
        &'__a__ mut self,
    ) -> <Self as HasExtractorMut>::ExtractorMut<'__a__> {
        match self {
            Self::Foo(value) => __PartialRefFooBar::Foo(value),
            Self::Bar(value) => __PartialRefFooBar::Bar(value),
        }
    }
}
impl FinalizeExtract for __PartialFooBar<IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<'__a__, __R__: MapTypeRef> FinalizeExtract
for __PartialRefFooBar<'__a__, __R__, IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<__F1__: MapType> ExtractField<Symbol!("Foo")>
for __PartialFooBar<IsPresent, __F1__> {
    type Value = u64;
    type Remainder = __PartialFooBar<IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Foo")>>::Value,
        <Self as ExtractField<Symbol!("Foo")>>::Remainder,
    > {
        match self {
            __PartialFooBar::Foo(value) => Ok(value),
            __PartialFooBar::Bar(value) => Err(__PartialFooBar::Bar(value)),
        }
    }
}
impl<__F0__: MapType> ExtractField<Symbol!("Bar")>
for __PartialFooBar<__F0__, IsPresent> {
    type Value = String;
    type Remainder = __PartialFooBar<__F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Bar")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Bar")>>::Value,
        <Self as ExtractField<Symbol!("Bar")>>::Remainder,
    > {
        match self {
            __PartialFooBar::Foo(value) => Err(__PartialFooBar::Foo(value)),
            __PartialFooBar::Bar(value) => Ok(value),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F1__: MapType> ExtractField<Symbol!("Foo")>
for __PartialRefFooBar<'__a__, __R__, IsPresent, __F1__> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, u64>;
    type Remainder = __PartialRefFooBar<'__a__, __R__, IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Foo")>>::Value,
        <Self as ExtractField<Symbol!("Foo")>>::Remainder,
    > {
        match self {
            __PartialRefFooBar::Foo(value) => Ok(value),
            __PartialRefFooBar::Bar(value) => Err(__PartialRefFooBar::Bar(value)),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType> ExtractField<Symbol!("Bar")>
for __PartialRefFooBar<'__a__, __R__, __F0__, IsPresent> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, String>;
    type Remainder = __PartialRefFooBar<'__a__, __R__, __F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Bar")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Bar")>>::Value,
        <Self as ExtractField<Symbol!("Bar")>>::Remainder,
    > {
        match self {
            __PartialRefFooBar::Foo(value) => Err(__PartialRefFooBar::Foo(value)),
            __PartialRefFooBar::Bar(value) => Ok(value),
        }
    }
}
pub enum FooBarBaz {
    Foo(u32),
    Bar(String),
    Baz(bool),
}
#[automatically_derived]
impl ::core::fmt::Debug for FooBarBaz {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Self::Foo(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Foo", &__self_0)
            }
            Self::Bar(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Bar", &__self_0)
            }
            Self::Baz(__self_0) => {
                ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Baz", &__self_0)
            }
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for FooBarBaz {}
#[automatically_derived]
impl ::core::cmp::PartialEq for FooBarBaz {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        ::core::intrinsics::discriminant_value(self)
            == ::core::intrinsics::discriminant_value(other)
            && match (self, other) {
                (Self::Foo(__self_0), Self::Foo(__arg1_0)) => __self_0 == __arg1_0,
                (Self::Bar(__self_0), Self::Bar(__arg1_0)) => __self_0 == __arg1_0,
                (Self::Baz(__self_0), Self::Baz(__arg1_0)) => __self_0 == __arg1_0,
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
impl HasFields for FooBarBaz {
    type Fields = Enum! { Foo(u32), Bar(String), Baz(bool) };
}
impl HasFieldsRef for FooBarBaz {
    type FieldsRef<'__a> = Enum! { Foo(&'__a u32), Bar(&'__a String), Baz(&'__a bool) }
    where
        Self: '__a;
}
impl FromFields for FooBarBaz {
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
                    Either::Right(rest) => {
                        match rest {
                            Either::Left(field) => {
                                let field = field.value;
                                Self::Baz(field)
                            }
                            Either::Right(rest) => match rest {}
                        }
                    }
                }
            }
        }
    }
}
impl ToFields for FooBarBaz {
    fn to_fields(self) -> <Self as HasFields>::Fields {
        match self {
            Self::Foo(field) => Either::Left(field.into()),
            Self::Bar(field) => Either::Right(Either::Left(field.into())),
            Self::Baz(field) => Either::Right(Either::Right(Either::Left(field.into()))),
        }
    }
}
impl ToFieldsRef for FooBarBaz {
    fn to_fields_ref<'__a>(&'__a self) -> <Self as HasFieldsRef>::FieldsRef<'__a>
    where
        Self: '__a,
    {
        match self {
            Self::Foo(field) => Either::Left(field.into()),
            Self::Bar(field) => Either::Right(Either::Left(field.into())),
            Self::Baz(field) => Either::Right(Either::Right(Either::Left(field.into()))),
        }
    }
}
impl FromVariant<Symbol!("Foo")> for FooBarBaz {
    type Value = u32;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
        value: <Self as FromVariant<Symbol!("Foo")>>::Value,
    ) -> Self {
        Self::Foo(value)
    }
}
impl FromVariant<Symbol!("Bar")> for FooBarBaz {
    type Value = String;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Bar")>,
        value: <Self as FromVariant<Symbol!("Bar")>>::Value,
    ) -> Self {
        Self::Bar(value)
    }
}
impl FromVariant<Symbol!("Baz")> for FooBarBaz {
    type Value = bool;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Baz")>,
        value: <Self as FromVariant<Symbol!("Baz")>>::Value,
    ) -> Self {
        Self::Baz(value)
    }
}
pub enum __PartialFooBarBaz<__F0__: MapType, __F1__: MapType, __F2__: MapType> {
    Foo(<__F0__ as MapType>::Map<u32>),
    Bar(<__F1__ as MapType>::Map<String>),
    Baz(<__F2__ as MapType>::Map<bool>),
}
pub enum __PartialRefFooBarBaz<
    '__a__,
    __R__: MapTypeRef,
    __F0__: MapType,
    __F1__: MapType,
    __F2__: MapType,
> {
    Foo(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, u32>>),
    Bar(<__F1__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, String>>),
    Baz(<__F2__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, bool>>),
}
impl<__F0__: MapType, __F1__: MapType, __F2__: MapType> PartialData
for __PartialFooBarBaz<__F0__, __F1__, __F2__> {
    type Target = FooBarBaz;
}
impl<
    '__a__,
    __R__: MapTypeRef,
    __F0__: MapType,
    __F1__: MapType,
    __F2__: MapType,
> PartialData for __PartialRefFooBarBaz<'__a__, __R__, __F0__, __F1__, __F2__> {
    type Target = FooBarBaz;
}
impl HasExtractor for FooBarBaz {
    type Extractor = __PartialFooBarBaz<IsPresent, IsPresent, IsPresent>;
    fn to_extractor(self) -> <Self as HasExtractor>::Extractor {
        match self {
            Self::Foo(value) => __PartialFooBarBaz::Foo(value),
            Self::Bar(value) => __PartialFooBarBaz::Bar(value),
            Self::Baz(value) => __PartialFooBarBaz::Baz(value),
        }
    }
    fn from_extractor(extractor: <Self as HasExtractor>::Extractor) -> Self {
        match extractor {
            __PartialFooBarBaz::Foo(value) => Self::Foo(value),
            __PartialFooBarBaz::Bar(value) => Self::Bar(value),
            __PartialFooBarBaz::Baz(value) => Self::Baz(value),
        }
    }
}
impl HasExtractorRef for FooBarBaz {
    type ExtractorRef<'__a__> = __PartialRefFooBarBaz<
        '__a__,
        IsRef,
        IsPresent,
        IsPresent,
        IsPresent,
    >
    where
        Self: '__a__;
    fn extractor_ref<'__a__>(
        &'__a__ self,
    ) -> <Self as HasExtractorRef>::ExtractorRef<'__a__> {
        match self {
            Self::Foo(value) => __PartialRefFooBarBaz::Foo(value),
            Self::Bar(value) => __PartialRefFooBarBaz::Bar(value),
            Self::Baz(value) => __PartialRefFooBarBaz::Baz(value),
        }
    }
}
impl HasExtractorMut for FooBarBaz {
    type ExtractorMut<'__a__> = __PartialRefFooBarBaz<
        '__a__,
        IsMut,
        IsPresent,
        IsPresent,
        IsPresent,
    >
    where
        Self: '__a__;
    fn extractor_mut<'__a__>(
        &'__a__ mut self,
    ) -> <Self as HasExtractorMut>::ExtractorMut<'__a__> {
        match self {
            Self::Foo(value) => __PartialRefFooBarBaz::Foo(value),
            Self::Bar(value) => __PartialRefFooBarBaz::Bar(value),
            Self::Baz(value) => __PartialRefFooBarBaz::Baz(value),
        }
    }
}
impl FinalizeExtract for __PartialFooBarBaz<IsVoid, IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<'__a__, __R__: MapTypeRef> FinalizeExtract
for __PartialRefFooBarBaz<'__a__, __R__, IsVoid, IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<__F1__: MapType, __F2__: MapType> ExtractField<Symbol!("Foo")>
for __PartialFooBarBaz<IsPresent, __F1__, __F2__> {
    type Value = u32;
    type Remainder = __PartialFooBarBaz<IsVoid, __F1__, __F2__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Foo")>>::Value,
        <Self as ExtractField<Symbol!("Foo")>>::Remainder,
    > {
        match self {
            __PartialFooBarBaz::Foo(value) => Ok(value),
            __PartialFooBarBaz::Bar(value) => Err(__PartialFooBarBaz::Bar(value)),
            __PartialFooBarBaz::Baz(value) => Err(__PartialFooBarBaz::Baz(value)),
        }
    }
}
impl<__F0__: MapType, __F2__: MapType> ExtractField<Symbol!("Bar")>
for __PartialFooBarBaz<__F0__, IsPresent, __F2__> {
    type Value = String;
    type Remainder = __PartialFooBarBaz<__F0__, IsVoid, __F2__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Bar")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Bar")>>::Value,
        <Self as ExtractField<Symbol!("Bar")>>::Remainder,
    > {
        match self {
            __PartialFooBarBaz::Foo(value) => Err(__PartialFooBarBaz::Foo(value)),
            __PartialFooBarBaz::Bar(value) => Ok(value),
            __PartialFooBarBaz::Baz(value) => Err(__PartialFooBarBaz::Baz(value)),
        }
    }
}
impl<__F0__: MapType, __F1__: MapType> ExtractField<Symbol!("Baz")>
for __PartialFooBarBaz<__F0__, __F1__, IsPresent> {
    type Value = bool;
    type Remainder = __PartialFooBarBaz<__F0__, __F1__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Baz")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Baz")>>::Value,
        <Self as ExtractField<Symbol!("Baz")>>::Remainder,
    > {
        match self {
            __PartialFooBarBaz::Foo(value) => Err(__PartialFooBarBaz::Foo(value)),
            __PartialFooBarBaz::Bar(value) => Err(__PartialFooBarBaz::Bar(value)),
            __PartialFooBarBaz::Baz(value) => Ok(value),
        }
    }
}
impl<
    '__a__,
    __R__: MapTypeRef,
    __F1__: MapType,
    __F2__: MapType,
> ExtractField<Symbol!("Foo")>
for __PartialRefFooBarBaz<'__a__, __R__, IsPresent, __F1__, __F2__> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, u32>;
    type Remainder = __PartialRefFooBarBaz<'__a__, __R__, IsVoid, __F1__, __F2__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Foo")>>::Value,
        <Self as ExtractField<Symbol!("Foo")>>::Remainder,
    > {
        match self {
            __PartialRefFooBarBaz::Foo(value) => Ok(value),
            __PartialRefFooBarBaz::Bar(value) => Err(__PartialRefFooBarBaz::Bar(value)),
            __PartialRefFooBarBaz::Baz(value) => Err(__PartialRefFooBarBaz::Baz(value)),
        }
    }
}
impl<
    '__a__,
    __R__: MapTypeRef,
    __F0__: MapType,
    __F2__: MapType,
> ExtractField<Symbol!("Bar")>
for __PartialRefFooBarBaz<'__a__, __R__, __F0__, IsPresent, __F2__> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, String>;
    type Remainder = __PartialRefFooBarBaz<'__a__, __R__, __F0__, IsVoid, __F2__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Bar")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Bar")>>::Value,
        <Self as ExtractField<Symbol!("Bar")>>::Remainder,
    > {
        match self {
            __PartialRefFooBarBaz::Foo(value) => Err(__PartialRefFooBarBaz::Foo(value)),
            __PartialRefFooBarBaz::Bar(value) => Ok(value),
            __PartialRefFooBarBaz::Baz(value) => Err(__PartialRefFooBarBaz::Baz(value)),
        }
    }
}
impl<
    '__a__,
    __R__: MapTypeRef,
    __F0__: MapType,
    __F1__: MapType,
> ExtractField<Symbol!("Baz")>
for __PartialRefFooBarBaz<'__a__, __R__, __F0__, __F1__, IsPresent> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, bool>;
    type Remainder = __PartialRefFooBarBaz<'__a__, __R__, __F0__, __F1__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Baz")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Baz")>>::Value,
        <Self as ExtractField<Symbol!("Baz")>>::Remainder,
    > {
        match self {
            __PartialRefFooBarBaz::Foo(value) => Err(__PartialRefFooBarBaz::Foo(value)),
            __PartialRefFooBarBaz::Bar(value) => Err(__PartialRefFooBarBaz::Bar(value)),
            __PartialRefFooBarBaz::Baz(value) => Ok(value),
        }
    }
}
fn main() {
    let _ = FooBar::Foo(1).upcast(PhantomData::<FooBarBaz>);
}
