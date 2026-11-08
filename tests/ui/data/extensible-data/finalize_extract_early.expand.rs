#![feature(prelude_import)]
//! Finalizing an extraction while a variant is still possible.
//!
//! `finalize_extract` on the remainder a `match` arm binds is an `E0599`, since only the
//! all-ruled-out extractor implements `FinalizeExtract`:
//! ``[CGP-E050] variant `Circle` of `Shape` is not extracted``.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct Circle {
    pub radius: f64,
}
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}
pub enum Shape {
    Circle(Circle),
    Rectangle(Rectangle),
}
impl HasFields for Shape {
    type Fields = Enum! { Circle(Circle), Rectangle(Rectangle) };
}
impl HasFieldsRef for Shape {
    type FieldsRef<'__a> = Enum! { Circle(&'__a Circle), Rectangle(&'__a Rectangle) }
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
            Either::Right(rest) => {
                match rest {
                    Either::Left(field) => {
                        let field = field.value;
                        Self::Rectangle(field)
                    }
                    Either::Right(rest) => match rest {}
                }
            }
        }
    }
}
impl ToFields for Shape {
    fn to_fields(self) -> <Self as HasFields>::Fields {
        match self {
            Self::Circle(field) => Either::Left(::core::convert::Into::into(field)),
            Self::Rectangle(field) => {
                Either::Right(Either::Left(::core::convert::Into::into(field)))
            }
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
            Self::Rectangle(field) => {
                Either::Right(Either::Left(::core::convert::Into::into(field)))
            }
        }
    }
}
impl FromVariant<Symbol!("Circle")> for Shape {
    type Value = Circle;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Circle")>,
        value: <Self as FromVariant<Symbol!("Circle")>>::Value,
    ) -> Self {
        Self::Circle(value)
    }
}
impl FromVariant<Symbol!("Rectangle")> for Shape {
    type Value = Rectangle;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Rectangle")>,
        value: <Self as FromVariant<Symbol!("Rectangle")>>::Value,
    ) -> Self {
        Self::Rectangle(value)
    }
}
///The partial extractor for [`Shape`].
pub enum __PartialShape<__F0__: MapType, __F1__: MapType> {
    Circle(<__F0__ as MapType>::Map<Circle>),
    Rectangle(<__F1__ as MapType>::Map<Rectangle>),
}
///The borrowed partial extractor for [`Shape`].
pub enum __PartialRefShape<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> {
    Circle(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, Circle>>),
    Rectangle(<__F1__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, Rectangle>>),
}
impl<__F0__: MapType, __F1__: MapType> PartialData for __PartialShape<__F0__, __F1__> {
    type Target = Shape;
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> PartialData
for __PartialRefShape<'__a__, __R__, __F0__, __F1__> {
    type Target = Shape;
}
impl HasExtractor for Shape {
    type Extractor = __PartialShape<IsPresent, IsPresent>;
    fn to_extractor(self) -> <Self as HasExtractor>::Extractor {
        match self {
            Self::Circle(value) => __PartialShape::Circle(value),
            Self::Rectangle(value) => __PartialShape::Rectangle(value),
        }
    }
    fn from_extractor(extractor: <Self as HasExtractor>::Extractor) -> Self {
        match extractor {
            __PartialShape::Circle(value) => Self::Circle(value),
            __PartialShape::Rectangle(value) => Self::Rectangle(value),
        }
    }
}
impl HasExtractorRef for Shape {
    type ExtractorRef<'__a__> = __PartialRefShape<'__a__, IsRef, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_ref<'__a__>(
        &'__a__ self,
    ) -> <Self as HasExtractorRef>::ExtractorRef<'__a__> {
        match self {
            Self::Circle(value) => __PartialRefShape::Circle(value),
            Self::Rectangle(value) => __PartialRefShape::Rectangle(value),
        }
    }
}
impl HasExtractorMut for Shape {
    type ExtractorMut<'__a__> = __PartialRefShape<'__a__, IsMut, IsPresent, IsPresent>
    where
        Self: '__a__;
    fn extractor_mut<'__a__>(
        &'__a__ mut self,
    ) -> <Self as HasExtractorMut>::ExtractorMut<'__a__> {
        match self {
            Self::Circle(value) => __PartialRefShape::Circle(value),
            Self::Rectangle(value) => __PartialRefShape::Rectangle(value),
        }
    }
}
impl FinalizeExtract for __PartialShape<IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<'__a__, __R__: MapTypeRef> FinalizeExtract
for __PartialRefShape<'__a__, __R__, IsVoid, IsVoid> {
    fn finalize_extract<__T__>(self) -> __T__ {
        match self {}
    }
}
impl<__F1__: MapType> ExtractField<Symbol!("Circle")>
for __PartialShape<IsPresent, __F1__> {
    type Value = Circle;
    type Remainder = __PartialShape<IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Circle")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Circle")>>::Value,
        <Self as ExtractField<Symbol!("Circle")>>::Remainder,
    > {
        match self {
            __PartialShape::Circle(value) => Ok(value),
            __PartialShape::Rectangle(value) => Err(__PartialShape::Rectangle(value)),
        }
    }
}
impl<__F0__: MapType> ExtractField<Symbol!("Rectangle")>
for __PartialShape<__F0__, IsPresent> {
    type Value = Rectangle;
    type Remainder = __PartialShape<__F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Rectangle")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Rectangle")>>::Value,
        <Self as ExtractField<Symbol!("Rectangle")>>::Remainder,
    > {
        match self {
            __PartialShape::Circle(value) => Err(__PartialShape::Circle(value)),
            __PartialShape::Rectangle(value) => Ok(value),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F1__: MapType> ExtractField<Symbol!("Circle")>
for __PartialRefShape<'__a__, __R__, IsPresent, __F1__> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, Circle>;
    type Remainder = __PartialRefShape<'__a__, __R__, IsVoid, __F1__>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Circle")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Circle")>>::Value,
        <Self as ExtractField<Symbol!("Circle")>>::Remainder,
    > {
        match self {
            __PartialRefShape::Circle(value) => Ok(value),
            __PartialRefShape::Rectangle(value) => {
                Err(__PartialRefShape::Rectangle(value))
            }
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType> ExtractField<Symbol!("Rectangle")>
for __PartialRefShape<'__a__, __R__, __F0__, IsPresent> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, Rectangle>;
    type Remainder = __PartialRefShape<'__a__, __R__, __F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Rectangle")>,
    ) -> Result<
        <Self as ExtractField<Symbol!("Rectangle")>>::Value,
        <Self as ExtractField<Symbol!("Rectangle")>>::Remainder,
    > {
        match self {
            __PartialRefShape::Circle(value) => Err(__PartialRefShape::Circle(value)),
            __PartialRefShape::Rectangle(value) => Ok(value),
        }
    }
}
fn main() {
    let shape = Shape::Circle(Circle { radius: 1.0 });
    let _: f64 = match shape
        .to_extractor()
        .extract_field(PhantomData::<Symbol!("Rectangle")>)
    {
        Ok(rect) => rect.width,
        Err(remainder) => remainder.finalize_extract(),
    };
}
