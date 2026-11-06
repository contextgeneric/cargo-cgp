#![feature(prelude_import)]
//! Downcasting into a target with a variant the source lacks.
//!
//! `FooBar` downcast into `FooQux { Foo, Qux }` needs every variant of `FooQux` in `FooBar`. The cast's
//! extractor recursion fails on `Qux`, which `FooBar` lacks, and the tool names it:
//! ``[CGP-E050] missing variant `Qux` on `FooBar` ``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::field::impls::CanDowncast;
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
                ::core::fmt::Formatter::debug_struct_field1_finish(f, "Foo", &__self_0)
            }
            Self::Bar(__self_0) => {
                ::core::fmt::Formatter::debug_struct_field1_finish(f, "Bar", &__self_0)
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
pub enum FooQux {
    Foo(u64),
    Qux(char),
}
#[automatically_derived]
impl ::core::fmt::Debug for FooQux {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match self {
            Self::Foo(__self_0) => {
                ::core::fmt::Formatter::debug_struct_field1_finish(f, "Foo", &__self_0)
            }
            Self::Qux(__self_0) => {
                ::core::fmt::Formatter::debug_struct_field1_finish(f, "Qux", &__self_0)
            }
        }
    }
}
#[automatically_derived]
impl ::core::marker::StructuralPartialEq for FooQux {}
#[automatically_derived]
impl ::core::cmp::PartialEq for FooQux {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        ::core::intrinsics::discriminant_value(self)
            == ::core::intrinsics::discriminant_value(other)
            && match (self, other) {
                (Self::Foo(__self_0), Self::Foo(__arg1_0)) => __self_0 == __arg1_0,
                (Self::Qux(__self_0), Self::Qux(__arg1_0)) => __self_0 == __arg1_0,
                _ => unsafe { ::core::intrinsics::unreachable() }
            }
    }
}
impl HasFields for FooQux {
    type Fields = Enum! { Foo(u64), Qux(char) };
}
impl HasFieldsRef for FooQux {
    type FieldsRef<'__a> = Enum! { Foo(&'__a u64), Qux(&'__a char) }
    where
        Self: '__a;
}
impl FromFields for FooQux {
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
                        Self::Qux(field)
                    }
                    Either::Right(rest) => match rest {}
                }
            }
        }
    }
}
impl ToFields for FooQux {
    fn to_fields(self) -> <Self as HasFields>::Fields {
        match self {
            Self::Foo(field) => Either::Left(field.into()),
            Self::Qux(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl ToFieldsRef for FooQux {
    fn to_fields_ref<'__a>(&'__a self) -> <Self as HasFieldsRef>::FieldsRef<'__a>
    where
        Self: '__a,
    {
        match self {
            Self::Foo(field) => Either::Left(field.into()),
            Self::Qux(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl FromVariant<Symbol!("Foo")> for FooQux {
    type Value = u64;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
        value: <Self as FromVariant<Symbol!("Foo")>>::Value,
    ) -> Self {
        Self::Foo(value)
    }
}
impl FromVariant<Symbol!("Qux")> for FooQux {
    type Value = char;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Qux")>,
        value: <Self as FromVariant<Symbol!("Qux")>>::Value,
    ) -> Self {
        Self::Qux(value)
    }
}
pub enum __PartialFooQux<__F0__: MapType, __F1__: MapType> {
    Foo(<__F0__ as MapType>::Map<u64>),
    Qux(<__F1__ as MapType>::Map<char>),
}
pub enum __PartialRefFooQux<
    '__a__,
    __R__: MapTypeRef,
    __F0__: MapType,
    __F1__: MapType,
> {
    Foo(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, u64>>),
    Qux(<__F1__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, char>>),
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialFooQux<__F0__, __F1__> {
    type Target = FooQux;
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> PartialData
for __PartialRefFooQux<'__a__, __R__, __F0__, __F1__> {
    type Target = FooQux;
}
impl HasExtractor for FooQux {
    type Extractor = __PartialFooQux<IsPresent, IsPresent>;
    fn to_extractor(self) -> <Self as HasExtractor>::Extractor {
        match self {
            Self::Foo(value) => __PartialFooQux::Foo(value),
            Self::Qux(value) => __PartialFooQux::Qux(value),
        }
    }
    fn from_extractor(extractor: <Self as HasExtractor>::Extractor) -> Self {
        match extractor {
            __PartialFooQux::Foo(value) => Self::Foo(value),
            __PartialFooQux::Qux(value) => Self::Qux(value),
        }
    }
}
impl HasExtractorRef for FooQux {
    type ExtractorRef<'__a__> = __PartialRefFooQux<'__a__, IsRef, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_ref<'__a__>(
        &'__a__ self,
    ) -> <Self as HasExtractorRef>::ExtractorRef<'__a__> {
        match self {
            Self::Foo(value) => __PartialRefFooQux::Foo(value),
            Self::Qux(value) => __PartialRefFooQux::Qux(value),
        }
    }
}
impl HasExtractorMut for FooQux {
    type ExtractorMut<'__a__> = __PartialRefFooQux<'__a__, IsMut, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_mut<'__a__>(
        &'__a__ mut self,
    ) -> <Self as HasExtractorMut>::ExtractorMut<'__a__> {
        match self {
            Self::Foo(value) => __PartialRefFooQux::Foo(value),
            Self::Qux(value) => __PartialRefFooQux::Qux(value),
        }
    }
}
impl FinalizeExtract for __PartialFooQux<IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<'__a__, __R__: MapTypeRef> FinalizeExtract
for __PartialRefFooQux<'__a__, __R__, IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<__F1__: MapType> ExtractField<Symbol!("Foo")>
for __PartialFooQux<IsPresent, __F1__> {
    type Value = u64;
    type Remainder = __PartialFooQux<IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Foo")>>::Value,
        <Self as ExtractField<Symbol!("Foo")>>::Remainder,
    > {
        match self {
            __PartialFooQux::Foo(value) => Ok(value),
            __PartialFooQux::Qux(value) => Err(__PartialFooQux::Qux(value)),
        }
    }
}
impl<__F0__: MapType> ExtractField<Symbol!("Qux")>
for __PartialFooQux<__F0__, IsPresent> {
    type Value = char;
    type Remainder = __PartialFooQux<__F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Qux")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Qux")>>::Value,
        <Self as ExtractField<Symbol!("Qux")>>::Remainder,
    > {
        match self {
            __PartialFooQux::Foo(value) => Err(__PartialFooQux::Foo(value)),
            __PartialFooQux::Qux(value) => Ok(value),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F1__: MapType> ExtractField<Symbol!("Foo")>
for __PartialRefFooQux<'__a__, __R__, IsPresent, __F1__> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, u64>;
    type Remainder = __PartialRefFooQux<'__a__, __R__, IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Foo")>>::Value,
        <Self as ExtractField<Symbol!("Foo")>>::Remainder,
    > {
        match self {
            __PartialRefFooQux::Foo(value) => Ok(value),
            __PartialRefFooQux::Qux(value) => Err(__PartialRefFooQux::Qux(value)),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType> ExtractField<Symbol!("Qux")>
for __PartialRefFooQux<'__a__, __R__, __F0__, IsPresent> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, char>;
    type Remainder = __PartialRefFooQux<'__a__, __R__, __F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Qux")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Qux")>>::Value,
        <Self as ExtractField<Symbol!("Qux")>>::Remainder,
    > {
        match self {
            __PartialRefFooQux::Foo(value) => Err(__PartialRefFooQux::Foo(value)),
            __PartialRefFooQux::Qux(value) => Ok(value),
        }
    }
}
fn main() {
    let _ = FooBar::Foo(1).downcast(PhantomData::<FooQux>);
}
