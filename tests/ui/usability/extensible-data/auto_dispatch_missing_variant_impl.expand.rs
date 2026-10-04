#![feature(prelude_import)]
//! A `#[cgp_auto_dispatch]` trait implemented for only one of an enum's two
//! payload types. The generated blanket impl requires every variant's payload to
//! implement the trait, so calling the method on the enum fails with `E0599` at
//! the call, whose notes list the matcher's unsatisfied `Computer` bound rather
//! than naming the payload, `Square`, that lacks the impl.
//!
//! The tool passes the error through as rustc wrote it, so a reader is told only
//! that `HasArea` exists on `Circle` and that a bound on the generated matcher
//! `MatchWithValueHandlersRef<ComputeArea>` failed. Naming the variant whose
//! payload lacks the impl is the presentation work outstanding.
//!
//! Error class: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md, with the headline under
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/error_codes/e0599.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct Circle;
pub struct Square;
pub enum Shape {
    Circle(Circle),
    Square(Square),
}
impl HasFields for Shape {
    type Fields = Sum![
        Field<Symbol!("Circle"), Circle>, Field<Symbol!("Square"), Square>
    ];
}
impl HasFieldsRef for Shape {
    type FieldsRef<'__a> = Sum![
        Field<Symbol!("Circle"), &'__a Circle>, Field<Symbol!("Square"), &'__a
        Square>
    ]
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
                        Self::Square(field)
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
            Self::Square(field) => Either::Right(Either::Left(field.into())),
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
            Self::Square(field) => Either::Right(Either::Left(field.into())),
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
impl FromVariant<Symbol!("Square")> for Shape {
    type Value = Square;
    fn from_variant(
        _tag: ::core::marker::PhantomData<Symbol!("Square")>,
        value: Self::Value,
    ) -> Self {
        Self::Square(value)
    }
}
pub enum __PartialShape<__F0__: MapType, __F1__: MapType> {
    Circle(<__F0__ as MapType>::Map<Circle>),
    Square(<__F1__ as MapType>::Map<Square>),
}
pub enum __PartialRefShape<'__a__, __R__: MapTypeRef, __F0__: MapType, __F1__: MapType> {
    Circle(<__F0__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, Circle>>),
    Square(<__F1__ as MapType>::Map<<__R__ as MapTypeRef>::Map<'__a__, Square>>),
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
            Self::Square(value) => __PartialShape::Square(value),
        }
    }
    fn from_extractor(extractor: Self::Extractor) -> Self {
        match extractor {
            __PartialShape::Circle(value) => Self::Circle(value),
            __PartialShape::Square(value) => Self::Square(value),
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
            Self::Square(value) => __PartialRefShape::Square(value),
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
            Self::Square(value) => __PartialRefShape::Square(value),
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
            __PartialShape::Square(value) => Err(__PartialShape::Square(value)),
        }
    }
}
impl<__F0__: MapType> ExtractField<Symbol!("Square")>
for __PartialShape<__F0__, IsPresent> {
    type Value = Square;
    type Remainder = __PartialShape<__F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Square")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialShape::Circle(value) => Err(__PartialShape::Circle(value)),
            __PartialShape::Square(value) => Ok(value),
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
            __PartialRefShape::Square(value) => Err(__PartialRefShape::Square(value)),
        }
    }
}
impl<'__a__, __R__: MapTypeRef, __F0__: MapType> ExtractField<Symbol!("Square")>
for __PartialRefShape<'__a__, __R__, __F0__, IsPresent> {
    type Value = <__R__ as MapTypeRef>::Map<'__a__, Square>;
    type Remainder = __PartialRefShape<'__a__, __R__, __F0__, IsVoid>;
    fn extract_field(
        self,
        _tag: ::core::marker::PhantomData<Symbol!("Square")>,
    ) -> Result<Self::Value, Self::Remainder> {
        match self {
            __PartialRefShape::Circle(value) => Err(__PartialRefShape::Circle(value)),
            __PartialRefShape::Square(value) => Ok(value),
        }
    }
}
pub trait HasArea {
    fn area(&self) -> f64;
}
impl<__Variants__> HasArea for __Variants__
where
    MatchWithValueHandlersRef<
        ComputeArea,
    >: for<'__a__> Computer<(), (), &'__a__ __Variants__, Output = f64>,
    __Variants__: HasExtractor,
{
    fn area(&self) -> f64 {
        <MatchWithValueHandlersRef<
            ComputeArea,
        > as Computer<_, _, _>>::compute(&(), ::core::marker::PhantomData::<()>, self)
    }
}
fn __compute_area__<'__a__, __Variants__: HasArea>(
    __Variants__: &'__a__ __Variants__,
) -> f64 {
    __Variants__.area()
}
impl<
    '__a__,
    __Variants__: HasArea,
    __Context__,
    __Code__,
> Computer<__Context__, __Code__, (&'__a__ __Variants__)> for ComputeArea {
    type Output = f64;
    fn compute(
        _context: &__Context__,
        _code: ::core::marker::PhantomData<__Code__>,
        (arg_0): (&'__a__ __Variants__),
    ) -> Self::Output {
        __compute_area__(arg_0)
    }
}
impl<
    '__a__,
    __Variants__: HasArea,
    __Context__,
    __Code__,
> IsProviderFor<ComputerComponent, __Context__, (__Code__, (&'__a__ __Variants__))>
for ComputeArea {}
pub struct ComputeArea;
impl DelegateComponent<ComputerRefComponent> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<ComputerRefComponent>,
{
    type Delegate = <PromoteComputer<
        Self,
    > as DelegateComponent<ComputerRefComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ComputerRefComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<ComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        ComputerRefComponent,
    >>::Delegate: IsProviderFor<ComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerComponent> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<TryComputerComponent>,
{
    type Delegate = <PromoteComputer<
        Self,
    > as DelegateComponent<TryComputerComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<TryComputerComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<TryComputerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        TryComputerComponent,
    >>::Delegate: IsProviderFor<TryComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerRefComponent> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<TryComputerRefComponent>,
{
    type Delegate = <PromoteComputer<
        Self,
    > as DelegateComponent<TryComputerRefComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<TryComputerRefComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<TryComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        TryComputerRefComponent,
    >>::Delegate: IsProviderFor<TryComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerComponent> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerComponent>,
{
    type Delegate = <PromoteComputer<
        Self,
    > as DelegateComponent<AsyncComputerComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<AsyncComputerComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerComponent,
    >>::Delegate: IsProviderFor<AsyncComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerRefComponent> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerRefComponent>,
{
    type Delegate = <PromoteComputer<
        Self,
    > as DelegateComponent<AsyncComputerRefComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerRefComponent,
    >>::Delegate: IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerComponent> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<HandlerComponent>,
{
    type Delegate = <PromoteComputer<
        Self,
    > as DelegateComponent<HandlerComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<HandlerComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<HandlerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        HandlerComponent,
    >>::Delegate: IsProviderFor<HandlerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerRefComponent> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<HandlerRefComponent>,
{
    type Delegate = <PromoteComputer<
        Self,
    > as DelegateComponent<HandlerRefComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<HandlerRefComponent, __Context__, __Params__> for ComputeArea
where
    PromoteComputer<Self>: DelegateComponent<HandlerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        HandlerRefComponent,
    >>::Delegate: IsProviderFor<HandlerRefComponent, __Context__, __Params__>,
{}
impl HasArea for Circle {
    fn area(&self) -> f64 {
        3.0
    }
}
fn main() {
    let _ = Shape::Circle(Circle).area();
}
