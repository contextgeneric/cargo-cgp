#![feature(prelude_import)]
//! An extensible-data cast the tool does not reshape, and the shredded spine in its help.
//!
//! `CanUpcast` widens one enum into another that must cover every variant of it. `Small`
//! has a `Bar` variant that `Big` lacks, so the cast cannot hold, and the mistake a reader
//! needs stated is exactly that: *`Small`'s `Bar` variant has no counterpart in `Big`*.
//!
//! The typed resolver declines the whole extensible-data family — casts, builders,
//! extractors — so this passes through as rustc wrote it. The reader is left with an
//! internal `FromVariant` bound, a caret on the *wrong* variant (`Big`'s `Foo`, the one
//! that does match), the macro-generated `__PartialSmall<IsVoid, IsPresent>` extractor
//! state they never wrote, and a hidden requirement. Reshaping this class into a coded
//! leaf naming the absent variant is the work outstanding.
//!
//! What the fixture *does* pin is the post-processing that survives a decline. rustc builds
//! its "similar impl" help out of styled fragments split at every difference between the two
//! traits, which shreds `Symbol<3, Chars<'B', …>>` into a fragment per character; read
//! fragment by fragment no CGP construct matches, so the help used to show the raw spine
//! while the main message beside it read `Symbol!("Bar")`. The fragments are now read as the
//! one line they render as, so both say `Symbol!("Bar")`.
//!
//! CGP error class: none yet — the extensible-data casts
//! (https://github.com/contextgeneric/cgp/blob/main/docs/concepts/extensible-variants.md)
//! are not in the upstream catalog.
//!
//! Issue: docs/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::field::impls::CanUpcast;
use cgp::prelude::*;
pub enum Small {
    Foo(u64),
    Bar(String),
}
impl HasFields for Small {
    type Fields = Sum![Field<Symbol!("Foo"), u64>, Field<Symbol!("Bar"), String>];
}
impl HasFieldsRef for Small {
    type FieldsRef<'__a> = Sum![
        Field<Symbol!("Foo"), &'__a u64>, Field<Symbol!("Bar"), &'__a String>
    ]
    where
        Self: '__a;
}
impl FromFields for Small {
    fn from_fields(rest: Self::Fields) -> Self {
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
impl ToFields for Small {
    fn to_fields(self) -> Self::Fields {
        match self {
            Self::Foo(field) => Either::Left(field.into()),
            Self::Bar(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl ToFieldsRef for Small {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        match self {
            Self::Foo(field) => Either::Left(field.into()),
            Self::Bar(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl FromVariant<Symbol!("Foo")> for Small {
    type Value = u64;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
        value: Self::Value,
    ) -> Self {
        Self::Foo(value)
    }
}
impl FromVariant<Symbol!("Bar")> for Small {
    type Value = String;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Bar")>,
        value: Self::Value,
    ) -> Self {
        Self::Bar(value)
    }
}
pub enum __PartialSmall<__F0__: MapType, __F1__: MapType> {
    Foo(<__F0__ as MapType>::Map<u64>),
    Bar(<__F1__ as MapType>::Map<String>),
}
pub enum __PartialRefSmall<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> {
    Foo(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, u64>>),
    Bar(<__F1__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, String>>),
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialSmall<__F0__, __F1__> {
    type Target = Small;
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> PartialData
for __PartialRefSmall<'__a__, __R__, __F0__, __F1__> {
    type Target = Small;
}
impl HasExtractor for Small {
    type Extractor = __PartialSmall<IsPresent, IsPresent>;
    fn to_extractor(self) -> Self::Extractor {
        match self {
            Self::Foo(value) => __PartialSmall::Foo(value),
            Self::Bar(value) => __PartialSmall::Bar(value),
        }
    }
    fn from_extractor(extractor: Self::Extractor) -> Self {
        match extractor {
            __PartialSmall::Foo(value) => Self::Foo(value),
            __PartialSmall::Bar(value) => Self::Bar(value),
        }
    }
}
impl HasExtractorRef for Small {
    type ExtractorRef<'__a__> = __PartialRefSmall<'__a__, IsRef, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_ref<'__a__>(&'__a__ self) -> Self::ExtractorRef<'__a__> {
        match self {
            Self::Foo(value) => __PartialRefSmall::Foo(value),
            Self::Bar(value) => __PartialRefSmall::Bar(value),
        }
    }
}
impl HasExtractorMut for Small {
    type ExtractorMut<'__a__> = __PartialRefSmall<'__a__, IsMut, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_mut<'__a__>(&'__a__ mut self) -> Self::ExtractorMut<'__a__> {
        match self {
            Self::Foo(value) => __PartialRefSmall::Foo(value),
            Self::Bar(value) => __PartialRefSmall::Bar(value),
        }
    }
}
impl FinalizeExtract for __PartialSmall<IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<'__a__, __R__: MapTypeRef> FinalizeExtract
for __PartialRefSmall<'__a__, __R__, IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<__F1__: MapType> ExtractField<Symbol!("Foo")>
for __PartialSmall<IsPresent, __F1__> {
    type Value = u64;
    type Remainder = __PartialSmall<IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialSmall::Foo(value) => Ok(value),
            __PartialSmall::Bar(value) => Err(__PartialSmall::Bar(value)),
        }
    }
}
impl<__F0__: MapType> ExtractField<Symbol!("Bar")>
for __PartialSmall<__F0__, IsPresent> {
    type Value = String;
    type Remainder = __PartialSmall<__F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Bar")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialSmall::Foo(value) => Err(__PartialSmall::Foo(value)),
            __PartialSmall::Bar(value) => Ok(value),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F1__: MapType> ExtractField<Symbol!("Foo")>
for __PartialRefSmall<'__a__, __R__, IsPresent, __F1__> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, u64>;
    type Remainder = __PartialRefSmall<'__a__, __R__, IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialRefSmall::Foo(value) => Ok(value),
            __PartialRefSmall::Bar(value) => Err(__PartialRefSmall::Bar(value)),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType> ExtractField<Symbol!("Bar")>
for __PartialRefSmall<'__a__, __R__, __F0__, IsPresent> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, String>;
    type Remainder = __PartialRefSmall<'__a__, __R__, __F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Bar")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialRefSmall::Foo(value) => Err(__PartialRefSmall::Foo(value)),
            __PartialRefSmall::Bar(value) => Ok(value),
        }
    }
}
pub enum Big {
    Foo(u64),
}
impl HasFields for Big {
    type Fields = Sum![Field<Symbol!("Foo"), u64>];
}
impl HasFieldsRef for Big {
    type FieldsRef<'__a> = Sum![Field<Symbol!("Foo"), &'__a u64>] where Self: '__a;
}
impl FromFields for Big {
    fn from_fields(rest: Self::Fields) -> Self {
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
    fn to_fields(self) -> Self::Fields {
        match self {
            Self::Foo(field) => Either::Left(field.into()),
        }
    }
}
impl ToFieldsRef for Big {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        match self {
            Self::Foo(field) => Either::Left(field.into()),
        }
    }
}
impl FromVariant<Symbol!("Foo")> for Big {
    type Value = u64;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Foo")>,
        value: Self::Value,
    ) -> Self {
        Self::Foo(value)
    }
}
pub enum __PartialBig<__F0__: MapType> {
    Foo(<__F0__ as MapType>::Map<u64>),
}
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
    fn to_extractor(self) -> Self::Extractor {
        match self {
            Self::Foo(value) => __PartialBig::Foo(value),
        }
    }
    fn from_extractor(extractor: Self::Extractor) -> Self {
        match extractor {
            __PartialBig::Foo(value) => Self::Foo(value),
        }
    }
}
impl HasExtractorRef for Big {
    type ExtractorRef<'__a__> = __PartialRefBig<'__a__, IsRef, IsPresent>
    where
        Self: '__a__;
    fn extractor_ref<'__a__>(&'__a__ self) -> Self::ExtractorRef<'__a__> {
        match self {
            Self::Foo(value) => __PartialRefBig::Foo(value),
        }
    }
}
impl HasExtractorMut for Big {
    type ExtractorMut<'__a__> = __PartialRefBig<'__a__, IsMut, IsPresent>
    where
        Self: '__a__;
    fn extractor_mut<'__a__>(&'__a__ mut self) -> Self::ExtractorMut<'__a__> {
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
    ) -> Result<Self::Value, Self::Remainder> {
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
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialRefBig::Foo(value) => Ok(value),
        }
    }
}
fn main() {
    let _: Big = Small::Foo(1).upcast(PhantomData::<Big>);
}
