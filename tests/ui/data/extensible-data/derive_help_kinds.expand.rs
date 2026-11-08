#![feature(prelude_import)]
//! Data operations on types that lack a data trait no derive can give them.
//!
//! `to_extractor` on the struct `Person` needs `HasExtractor`, whose derive `ExtractField` takes
//! an enum; `builder` on the enum `Shape` needs `HasBuilder`, whose derive `BuildField` takes a
//! struct; and `upcast` from `Option<u64>` needs `HasFields` and `HasExtractor` of a type the crate
//! does not define. Each block states the missing trait, and none names a derive to add.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::field::impls::CanUpcast;
use cgp::prelude::*;
pub struct Person {
    pub name: String,
}
impl HasField<Symbol!("name")> for Person {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("name")>,
    ) -> &Self::Value {
        &self.name
    }
}
impl HasFieldMut<Symbol!("name")> for Person {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("name")>,
    ) -> &mut Self::Value {
        &mut self.name
    }
}
pub enum Shape {
    Circle(f64),
}
impl HasFields for Shape {
    type Fields = Enum! { Circle(f64) };
}
impl HasFieldsRef for Shape {
    type FieldsRef<'__a> = Enum! { Circle(&'__a f64) }
    where
        Self: '__a;
}
impl FromFields for Shape {
    fn from_fields(rest: <Self as HasFields>::Fields) -> Self {
        match rest {
            Either::Left(field) => {
                let field = field.value;
                Self::Circle(field)
            }
            Either::Right(rest) => match rest {}
        }
    }
}
impl ToFields for Shape {
    fn to_fields(self) -> <Self as HasFields>::Fields {
        match self {
            Self::Circle(field) => Either::Left(::core::convert::Into::into(field)),
        }
    }
}
impl ToFieldsRef for Shape {
    fn to_fields_ref<'__a>(&'__a self) -> <Self as HasFieldsRef>::FieldsRef<'__a>
    where
        Self: '__a,
    {
        match self {
            Self::Circle(field) => Either::Left(::core::convert::Into::into(field)),
        }
    }
}
pub enum Big {
    Foo(u64),
}
impl HasFields for Big {
    type Fields = Enum! { Foo(u64) };
}
impl HasFieldsRef for Big {
    type FieldsRef<'__a> = Enum! { Foo(&'__a u64) }
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
            Either::Right(rest) => match rest {}
        }
    }
}
impl ToFields for Big {
    fn to_fields(self) -> <Self as HasFields>::Fields {
        match self {
            Self::Foo(field) => Either::Left(::core::convert::Into::into(field)),
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
///The partial extractor for [`Big`].
pub enum __PartialBig<__F0__: MapType> {
    Foo(<__F0__ as MapType>::Map<u64>),
}
///The borrowed partial extractor for [`Big`].
pub enum __PartialRefBig<'__a__, __R__: MapTypeRef, __F0__: MapType> {
    Foo(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, u64>>),
}
impl<__F0__: MapType> PartialData for __PartialBig<__F0__> {
    type Target = Big;
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType> PartialData
for __PartialRefBig<'__a__, __R__, __F0__> {
    type Target = Big;
}
impl HasExtractor for Big {
    type Extractor = __PartialBig<IsPresent>;
    fn to_extractor(self) -> <Self as HasExtractor>::Extractor {
        match self {
            Self::Foo(value) => __PartialBig::Foo(value),
        }
    }
    fn from_extractor(extractor: <Self as HasExtractor>::Extractor) -> Self {
        match extractor {
            __PartialBig::Foo(value) => Self::Foo(value),
        }
    }
}
impl HasExtractorRef for Big {
    type ExtractorRef<'__a__> = __PartialRefBig<'__a__, IsRef, IsPresent>
    where
        Self: '__a__;
    fn extractor_ref<'__a__>(
        &'__a__ self,
    ) -> <Self as HasExtractorRef>::ExtractorRef<'__a__> {
        match self {
            Self::Foo(value) => __PartialRefBig::Foo(value),
        }
    }
}
impl HasExtractorMut for Big {
    type ExtractorMut<'__a__> = __PartialRefBig<'__a__, IsMut, IsPresent>
    where
        Self: '__a__;
    fn extractor_mut<'__a__>(
        &'__a__ mut self,
    ) -> <Self as HasExtractorMut>::ExtractorMut<'__a__> {
        match self {
            Self::Foo(value) => __PartialRefBig::Foo(value),
        }
    }
}
impl FinalizeExtract for __PartialBig<IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<'__a__, __R__: MapTypeRef> FinalizeExtract
for __PartialRefBig<'__a__, __R__, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl ExtractField<Symbol!("Foo")> for __PartialBig<IsPresent> {
    type Value = u64;
    type Remainder = __PartialBig<IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Foo")>>::Value,
        <Self as ExtractField<Symbol!("Foo")>>::Remainder,
    > {
        match self {
            __PartialBig::Foo(value) => Ok(value),
        }
    }
}
impl<'__a__, __R__: MapTypeRef> ExtractField<Symbol!("Foo")>
for __PartialRefBig<'__a__, __R__, IsPresent> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, u64>;
    type Remainder = __PartialRefBig<'__a__, __R__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Foo")>>::Value,
        <Self as ExtractField<Symbol!("Foo")>>::Remainder,
    > {
        match self {
            __PartialRefBig::Foo(value) => Ok(value),
        }
    }
}
pub fn extract_person(person: Person) {
    let _ = person.to_extractor();
}
pub fn build_shape() {
    let _ = Shape::builder();
}
pub fn upcast_option() {
    let value: Option<u64> = None;
    let _: Big = value.upcast(PhantomData::<Big>);
}
fn main() {}
