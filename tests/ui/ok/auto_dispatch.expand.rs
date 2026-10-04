#![feature(prelude_import)]
//! `#[cgp_auto_dispatch]` across the method shapes its expansion distinguishes:
//! a `&self` method, a `&mut self` method taking an argument, a by-value `self`
//! method, a method borrowing through an elided lifetime, and an async method
//! stacked with `#[async_trait]`. The program compiles clean, so the snapshot of
//! cargo-cgp's output is empty; the `.expand.rs` pins the enum-level blanket
//! impls and the per-variant computers the macro generates, down to the provider
//! impls their nested `#[cgp_computer]` expands to.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct Circle {
    pub radius: f64,
}
pub struct Square {
    pub side: f64,
}
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
pub trait HasShape {
    fn area(&self) -> f64;
    fn scale(&mut self, factor: f64);
    fn into_name(self) -> &'static str;
    fn label(&self, prefix: &str) -> String;
}
impl<__Variants__> HasShape for __Variants__
where
    MatchWithValueHandlersRef<
        ComputeArea,
    >: for<'__a__> Computer<(), (), &'__a__ __Variants__, Output = f64>,
    MatchFirstWithValueHandlersMut<
        ComputeScale,
    >: for<'__a__> Computer<(), (), (&'__a__ mut __Variants__, (f64)), Output = ()>,
    MatchWithValueHandlers<
        ComputeIntoName,
    >: Computer<(), (), __Variants__, Output = &'static str>,
    MatchFirstWithValueHandlersRef<
        ComputeLabel,
    >: for<'__a__> Computer<
        (),
        (),
        (&'__a__ __Variants__, (&'__a__ str)),
        Output = String,
    >,
    __Variants__: HasExtractor,
{
    fn area(&self) -> f64 {
        <MatchWithValueHandlersRef<
            ComputeArea,
        > as Computer<_, _, _>>::compute(&(), ::core::marker::PhantomData::<()>, self)
    }
    fn scale(&mut self, arg_0: f64) {
        <MatchFirstWithValueHandlersMut<
            ComputeScale,
        > as Computer<
            _,
            _,
            _,
        >>::compute(&(), ::core::marker::PhantomData::<()>, (self, (arg_0)))
    }
    fn into_name(self) -> &'static str {
        <MatchWithValueHandlers<
            ComputeIntoName,
        > as Computer<_, _, _>>::compute(&(), ::core::marker::PhantomData::<()>, self)
    }
    fn label(&self, arg_0: &str) -> String {
        <MatchFirstWithValueHandlersRef<
            ComputeLabel,
        > as Computer<
            _,
            _,
            _,
        >>::compute(&(), ::core::marker::PhantomData::<()>, (self, (arg_0)))
    }
}
fn __compute_area__<'__a__, __Variants__: HasShape>(
    __Variants__: &'__a__ __Variants__,
) -> f64 {
    __Variants__.area()
}
impl<
    '__a__,
    __Variants__: HasShape,
    __Context__,
    __Code__,
> Computer<__Context__, __Code__, (&'__a__ __Variants__)> for ComputeArea {
    type Output = f64;
    fn compute(
        _context: &__Context__,
        _code: PhantomData<__Code__>,
        (arg_0): (&'__a__ __Variants__),
    ) -> Self::Output {
        __compute_area__(arg_0)
    }
}
impl<
    '__a__,
    __Variants__: HasShape,
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
fn __compute_scale__<'__a__, __Variants__: HasShape>(
    __Variants__: &'__a__ mut __Variants__,
    (arg_0): (f64),
) {
    __Variants__.scale(arg_0)
}
impl<
    '__a__,
    __Variants__: HasShape,
    __Context__,
    __Code__,
> Computer<__Context__, __Code__, (&'__a__ mut __Variants__, (f64))> for ComputeScale {
    type Output = ();
    fn compute(
        _context: &__Context__,
        _code: PhantomData<__Code__>,
        (arg_0, arg_1): (&'__a__ mut __Variants__, (f64)),
    ) -> Self::Output {
        __compute_scale__(arg_0, arg_1)
    }
}
impl<
    '__a__,
    __Variants__: HasShape,
    __Context__,
    __Code__,
> IsProviderFor<
    ComputerComponent,
    __Context__,
    (__Code__, (&'__a__ mut __Variants__, (f64))),
> for ComputeScale {}
pub struct ComputeScale;
impl DelegateComponent<ComputerRefComponent> for ComputeScale
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
> IsProviderFor<ComputerRefComponent, __Context__, __Params__> for ComputeScale
where
    PromoteComputer<Self>: DelegateComponent<ComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        ComputerRefComponent,
    >>::Delegate: IsProviderFor<ComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerComponent> for ComputeScale
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
> IsProviderFor<TryComputerComponent, __Context__, __Params__> for ComputeScale
where
    PromoteComputer<Self>: DelegateComponent<TryComputerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        TryComputerComponent,
    >>::Delegate: IsProviderFor<TryComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerRefComponent> for ComputeScale
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
> IsProviderFor<TryComputerRefComponent, __Context__, __Params__> for ComputeScale
where
    PromoteComputer<Self>: DelegateComponent<TryComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        TryComputerRefComponent,
    >>::Delegate: IsProviderFor<TryComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerComponent> for ComputeScale
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
> IsProviderFor<AsyncComputerComponent, __Context__, __Params__> for ComputeScale
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerComponent,
    >>::Delegate: IsProviderFor<AsyncComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerRefComponent> for ComputeScale
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
> IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__> for ComputeScale
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerRefComponent,
    >>::Delegate: IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerComponent> for ComputeScale
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
> IsProviderFor<HandlerComponent, __Context__, __Params__> for ComputeScale
where
    PromoteComputer<Self>: DelegateComponent<HandlerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        HandlerComponent,
    >>::Delegate: IsProviderFor<HandlerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerRefComponent> for ComputeScale
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
> IsProviderFor<HandlerRefComponent, __Context__, __Params__> for ComputeScale
where
    PromoteComputer<Self>: DelegateComponent<HandlerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        HandlerRefComponent,
    >>::Delegate: IsProviderFor<HandlerRefComponent, __Context__, __Params__>,
{}
fn __compute_into_name__<__Variants__: HasShape>(
    __Variants__: __Variants__,
) -> &'static str {
    __Variants__.into_name()
}
impl<
    __Variants__: HasShape,
    __Context__,
    __Code__,
> Computer<__Context__, __Code__, (__Variants__)> for ComputeIntoName {
    type Output = &'static str;
    fn compute(
        _context: &__Context__,
        _code: PhantomData<__Code__>,
        (arg_0): (__Variants__),
    ) -> Self::Output {
        __compute_into_name__(arg_0)
    }
}
impl<
    __Variants__: HasShape,
    __Context__,
    __Code__,
> IsProviderFor<ComputerComponent, __Context__, (__Code__, (__Variants__))>
for ComputeIntoName {}
pub struct ComputeIntoName;
impl DelegateComponent<ComputerRefComponent> for ComputeIntoName
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
> IsProviderFor<ComputerRefComponent, __Context__, __Params__> for ComputeIntoName
where
    PromoteComputer<Self>: DelegateComponent<ComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        ComputerRefComponent,
    >>::Delegate: IsProviderFor<ComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerComponent> for ComputeIntoName
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
> IsProviderFor<TryComputerComponent, __Context__, __Params__> for ComputeIntoName
where
    PromoteComputer<Self>: DelegateComponent<TryComputerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        TryComputerComponent,
    >>::Delegate: IsProviderFor<TryComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerRefComponent> for ComputeIntoName
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
> IsProviderFor<TryComputerRefComponent, __Context__, __Params__> for ComputeIntoName
where
    PromoteComputer<Self>: DelegateComponent<TryComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        TryComputerRefComponent,
    >>::Delegate: IsProviderFor<TryComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerComponent> for ComputeIntoName
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
> IsProviderFor<AsyncComputerComponent, __Context__, __Params__> for ComputeIntoName
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerComponent,
    >>::Delegate: IsProviderFor<AsyncComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerRefComponent> for ComputeIntoName
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
> IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__> for ComputeIntoName
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerRefComponent,
    >>::Delegate: IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerComponent> for ComputeIntoName
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
> IsProviderFor<HandlerComponent, __Context__, __Params__> for ComputeIntoName
where
    PromoteComputer<Self>: DelegateComponent<HandlerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        HandlerComponent,
    >>::Delegate: IsProviderFor<HandlerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerRefComponent> for ComputeIntoName
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
> IsProviderFor<HandlerRefComponent, __Context__, __Params__> for ComputeIntoName
where
    PromoteComputer<Self>: DelegateComponent<HandlerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        HandlerRefComponent,
    >>::Delegate: IsProviderFor<HandlerRefComponent, __Context__, __Params__>,
{}
fn __compute_label__<'__a__, __Variants__: HasShape>(
    __Variants__: &'__a__ __Variants__,
    (arg_0): (&'__a__ str),
) -> String {
    __Variants__.label(arg_0)
}
impl<
    '__a__,
    __Variants__: HasShape,
    __Context__,
    __Code__,
> Computer<__Context__, __Code__, (&'__a__ __Variants__, (&'__a__ str))>
for ComputeLabel {
    type Output = String;
    fn compute(
        _context: &__Context__,
        _code: PhantomData<__Code__>,
        (arg_0, arg_1): (&'__a__ __Variants__, (&'__a__ str)),
    ) -> Self::Output {
        __compute_label__(arg_0, arg_1)
    }
}
impl<
    '__a__,
    __Variants__: HasShape,
    __Context__,
    __Code__,
> IsProviderFor<
    ComputerComponent,
    __Context__,
    (__Code__, (&'__a__ __Variants__, (&'__a__ str))),
> for ComputeLabel {}
pub struct ComputeLabel;
impl DelegateComponent<ComputerRefComponent> for ComputeLabel
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
> IsProviderFor<ComputerRefComponent, __Context__, __Params__> for ComputeLabel
where
    PromoteComputer<Self>: DelegateComponent<ComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        ComputerRefComponent,
    >>::Delegate: IsProviderFor<ComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerComponent> for ComputeLabel
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
> IsProviderFor<TryComputerComponent, __Context__, __Params__> for ComputeLabel
where
    PromoteComputer<Self>: DelegateComponent<TryComputerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        TryComputerComponent,
    >>::Delegate: IsProviderFor<TryComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TryComputerRefComponent> for ComputeLabel
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
> IsProviderFor<TryComputerRefComponent, __Context__, __Params__> for ComputeLabel
where
    PromoteComputer<Self>: DelegateComponent<TryComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        TryComputerRefComponent,
    >>::Delegate: IsProviderFor<TryComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerComponent> for ComputeLabel
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
> IsProviderFor<AsyncComputerComponent, __Context__, __Params__> for ComputeLabel
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerComponent,
    >>::Delegate: IsProviderFor<AsyncComputerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<AsyncComputerRefComponent> for ComputeLabel
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
> IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__> for ComputeLabel
where
    PromoteComputer<Self>: DelegateComponent<AsyncComputerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerRefComponent,
    >>::Delegate: IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerComponent> for ComputeLabel
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
> IsProviderFor<HandlerComponent, __Context__, __Params__> for ComputeLabel
where
    PromoteComputer<Self>: DelegateComponent<HandlerComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        HandlerComponent,
    >>::Delegate: IsProviderFor<HandlerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerRefComponent> for ComputeLabel
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
> IsProviderFor<HandlerRefComponent, __Context__, __Params__> for ComputeLabel
where
    PromoteComputer<Self>: DelegateComponent<HandlerRefComponent>,
    <PromoteComputer<
        Self,
    > as DelegateComponent<
        HandlerRefComponent,
    >>::Delegate: IsProviderFor<HandlerRefComponent, __Context__, __Params__>,
{}
pub trait CanDescribe {
    fn describe(&self) -> impl ::core::future::Future<Output = String>;
}
impl<__Variants__> CanDescribe for __Variants__
where
    MatchWithValueHandlersRef<
        ComputeDescribe,
    >: for<'__a__> AsyncComputer<(), (), &'__a__ __Variants__, Output = String>,
    __Variants__: HasExtractor,
{
    async fn describe(&self) -> String {
        <MatchWithValueHandlersRef<
            ComputeDescribe,
        > as AsyncComputer<
            _,
            _,
            _,
        >>::compute_async(&(), ::core::marker::PhantomData::<()>, self)
            .await
    }
}
async fn __compute_describe__<'__a__, __Variants__: CanDescribe>(
    __Variants__: &'__a__ __Variants__,
) -> String {
    __Variants__.describe().await
}
impl<
    '__a__,
    __Variants__: CanDescribe,
    __Context__,
    __Code__,
> AsyncComputer<__Context__, __Code__, (&'__a__ __Variants__)> for ComputeDescribe {
    type Output = String;
    async fn compute_async(
        _context: &__Context__,
        _code: PhantomData<__Code__>,
        (arg_0): (&'__a__ __Variants__),
    ) -> Self::Output {
        __compute_describe__(arg_0).await
    }
}
impl<
    '__a__,
    __Variants__: CanDescribe,
    __Context__,
    __Code__,
> IsProviderFor<AsyncComputerComponent, __Context__, (__Code__, (&'__a__ __Variants__))>
for ComputeDescribe {}
pub struct ComputeDescribe;
impl DelegateComponent<AsyncComputerRefComponent> for ComputeDescribe
where
    PromoteAsyncComputer<Self>: DelegateComponent<AsyncComputerRefComponent>,
{
    type Delegate = <PromoteAsyncComputer<
        Self,
    > as DelegateComponent<AsyncComputerRefComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__> for ComputeDescribe
where
    PromoteAsyncComputer<Self>: DelegateComponent<AsyncComputerRefComponent>,
    <PromoteAsyncComputer<
        Self,
    > as DelegateComponent<
        AsyncComputerRefComponent,
    >>::Delegate: IsProviderFor<AsyncComputerRefComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerComponent> for ComputeDescribe
where
    PromoteAsyncComputer<Self>: DelegateComponent<HandlerComponent>,
{
    type Delegate = <PromoteAsyncComputer<
        Self,
    > as DelegateComponent<HandlerComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<HandlerComponent, __Context__, __Params__> for ComputeDescribe
where
    PromoteAsyncComputer<Self>: DelegateComponent<HandlerComponent>,
    <PromoteAsyncComputer<
        Self,
    > as DelegateComponent<
        HandlerComponent,
    >>::Delegate: IsProviderFor<HandlerComponent, __Context__, __Params__>,
{}
impl DelegateComponent<HandlerRefComponent> for ComputeDescribe
where
    PromoteAsyncComputer<Self>: DelegateComponent<HandlerRefComponent>,
{
    type Delegate = <PromoteAsyncComputer<
        Self,
    > as DelegateComponent<HandlerRefComponent>>::Delegate;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<HandlerRefComponent, __Context__, __Params__> for ComputeDescribe
where
    PromoteAsyncComputer<Self>: DelegateComponent<HandlerRefComponent>,
    <PromoteAsyncComputer<
        Self,
    > as DelegateComponent<
        HandlerRefComponent,
    >>::Delegate: IsProviderFor<HandlerRefComponent, __Context__, __Params__>,
{}
impl HasShape for Circle {
    fn area(&self) -> f64 {
        3.0 * self.radius * self.radius
    }
    fn scale(&mut self, factor: f64) {
        self.radius *= factor;
    }
    fn into_name(self) -> &'static str {
        "circle"
    }
    fn label(&self, prefix: &str) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("{0} circle", prefix))
        })
    }
}
impl HasShape for Square {
    fn area(&self) -> f64 {
        self.side * self.side
    }
    fn scale(&mut self, factor: f64) {
        self.side *= factor;
    }
    fn into_name(self) -> &'static str {
        "square"
    }
    fn label(&self, prefix: &str) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("{0} square", prefix))
        })
    }
}
impl CanDescribe for Circle {
    async fn describe(&self) -> String {
        "a circle".to_owned()
    }
}
impl CanDescribe for Square {
    async fn describe(&self) -> String {
        "a square".to_owned()
    }
}
fn main() {
    let mut shape = Shape::Square(Square { side: 2.0 });
    shape.scale(2.0);
    {
        match (&shape.area(), &16.0) {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    {
        match (&shape.label("big"), &"big square") {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
    let _ = shape.describe();
    {
        match (&shape.into_name(), &"square") {
            (left_val, right_val) => {
                if !(*left_val == *right_val) {
                    let kind = ::core::panicking::AssertKind::Eq;
                    ::core::panicking::assert_failed(
                        kind,
                        &*left_val,
                        &*right_val,
                        ::core::option::Option::None,
                    );
                }
            }
        }
    };
}
