#![feature(prelude_import)]
//! A `MatchWithHandlers` list missing the handler for one variant.
//!
//! An `E0277` on `__PartialShape<IsVoid, IsPresent>: FinalizeExtract`, which names no missing
//! variant. rustc reports it twice at one caret, and the tool merges the repeat into the first
//! block.
//!
//! CGP error class: none yet.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::marker::PhantomData;
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
    fn from_fields(rest: Self::Fields) -> Self {
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
    fn to_fields(self) -> Self::Fields {
        match self {
            Self::Circle(field) => Either::Left(field.into()),
            Self::Rectangle(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl ToFieldsRef for Shape {
    fn to_fields_ref<'__a>(&'__a self) -> Self::FieldsRef<'__a>
    where
        Self: '__a,
    {
        match self {
            Self::Circle(field) => Either::Left(field.into()),
            Self::Rectangle(field) => Either::Right(Either::Left(field.into())),
        }
    }
}
impl FromVariant<Symbol!("Circle")> for Shape {
    type Value = Circle;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Circle")>,
        value: Self::Value,
    ) -> Self {
        Self::Circle(value)
    }
}
impl FromVariant<Symbol!("Rectangle")> for Shape {
    type Value = Rectangle;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Rectangle")>,
        value: Self::Value,
    ) -> Self {
        Self::Rectangle(value)
    }
}
pub enum __PartialShape<__F0__: MapType, __F1__: MapType> {
    Circle(<__F0__ as MapType>::Map<Circle>),
    Rectangle(<__F1__ as MapType>::Map<Rectangle>),
}
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
    fn to_extractor(self) -> Self::Extractor {
        match self {
            Self::Circle(value) => __PartialShape::Circle(value),
            Self::Rectangle(value) => __PartialShape::Rectangle(value),
        }
    }
    fn from_extractor(extractor: Self::Extractor) -> Self {
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
    fn extractor_ref<'__a__>(&'__a__ self) -> Self::ExtractorRef<'__a__> {
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
    fn extractor_mut<'__a__>(&'__a__ mut self) -> Self::ExtractorMut<'__a__> {
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
    ) -> Result<Self::Value, Self::Remainder> {
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
    ) -> Result<Self::Value, Self::Remainder> {
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
    ) -> Result<Self::Value, Self::Remainder> {
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
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialRefShape::Circle(value) => Err(__PartialRefShape::Circle(value)),
            __PartialRefShape::Rectangle(value) => Ok(value),
        }
    }
}
pub struct ComputeArea;
impl<Context, Code> Computer<Context, Code, Circle> for ComputeArea {
    type Output = f64;
    fn compute(_context: &Context, _code: PhantomData<Code>, shape: Circle) -> f64 {
        shape.radius
    }
}
impl<Context, Code> IsProviderFor<ComputerComponent, Context, (Code, Circle)>
for ComputeArea {}
impl<Context, Code> Computer<Context, Code, Rectangle> for ComputeArea {
    type Output = f64;
    fn compute(_context: &Context, _code: PhantomData<Code>, shape: Rectangle) -> f64 {
        shape.width * shape.height
    }
}
impl<Context, Code> IsProviderFor<ComputerComponent, Context, (Code, Rectangle)>
for ComputeArea {}
use cgp::extra::dispatch::{ExtractFieldAndHandle, HandleFieldValue, MatchWithHandlers};
fn area(shape: Shape) -> f64 {
    MatchWithHandlers::<
        Product![
            ExtractFieldAndHandle<Symbol!("Circle"), HandleFieldValue<ComputeArea>>
        ],
    >::compute(&(), PhantomData::<()>, shape)
}
fn main() {}
