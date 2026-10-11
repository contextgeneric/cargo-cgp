#![feature(prelude_import)]
//! A compiler crash after a buffered diagnostic keeps rustc's output order.
//!
//! The program has a CGP wiring failure (`Rectangle` lacks the `height` field `RectangleArea`
//! reads) and then a plain type error. `-Ztreat-err-as-bug=2` makes the compiler panic on the
//! second error, as an internal compiler error. The emitter holds the wiring failure in its buffer
//! until the end of the compilation, so the driver's panic hook prints the buffer, untransformed,
//! before rustc's crash report: the wiring failure comes first, as plain rustc prints it. Without
//! the hook, the crash report would come first and the buffered errors after it.
//!
//! `RUSTC_ICE=0` stops rustc writing a crash-report file into the fixture's crate, and
//! `RUST_BACKTRACE=0` keeps the backtrace out of the snapshot.
//!
//! Error class: none; this pins how the tool degrades when the compiler itself crashes.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanCalculateArea {
    fn area(&self) -> f64;
}
impl<__Context__> CanCalculateArea for __Context__
where
    __Context__: AreaCalculator<__Context__>,
{
    fn area(&self) -> f64 {
        __Context__::area(self)
    }
}
pub trait AreaCalculator<
    __Context__,
>: IsProviderFor<AreaCalculatorComponent, __Context__, ()> {
    fn area(__context__: &__Context__) -> f64;
}
impl<__Provider__, __Context__> AreaCalculator<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<AreaCalculatorComponent>
        + IsProviderFor<AreaCalculatorComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        AreaCalculatorComponent,
    >>::Delegate: AreaCalculator<__Context__>,
{
    fn area(__context__: &__Context__) -> f64 {
        <__Provider__ as DelegateComponent<
            AreaCalculatorComponent,
        >>::Delegate::area(__context__)
    }
}
pub struct AreaCalculatorComponent;
impl<__Context__> AreaCalculator<__Context__> for UseContext
where
    __Context__: CanCalculateArea,
{
    fn area(__context__: &__Context__) -> f64 {
        __Context__::area(__context__)
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, ()> for UseContext
where
    __Context__: CanCalculateArea,
{}
impl<__Context__, __Components__, __Path__> AreaCalculator<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: AreaCalculator<__Context__>,
{
    fn area(__context__: &__Context__) -> f64 {
        <__Components__ as DelegateComponent<__Path__>>::Delegate::area(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<AreaCalculatorComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<AreaCalculatorComponent, __Context__, ()>
        + AreaCalculator<__Context__>,
{}
pub trait HasRectangleFields {
    fn width(&self) -> f64;
    fn height(&self) -> f64;
}
impl<__Context__> HasRectangleFields for __Context__
where
    __Context__: HasField<Symbol!("width"), Value = f64>,
    __Context__: HasField<Symbol!("height"), Value = f64>,
{
    fn width(&self) -> f64 {
        self.get_field(::core::marker::PhantomData::<Symbol!("width")>).clone()
    }
    fn height(&self) -> f64 {
        self.get_field(::core::marker::PhantomData::<Symbol!("height")>).clone()
    }
}
impl<__Context__> AreaCalculator<__Context__> for RectangleArea
where
    __Context__: HasRectangleFields,
{
    fn area(__context__: &__Context__) -> f64 {
        __context__.width() * __context__.height()
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, ()>
for RectangleArea
where
    __Context__: HasRectangleFields,
{}
pub struct RectangleArea;
pub struct Rectangle {
    pub width: f64,
}
impl HasField<Symbol!("width")> for Rectangle {
    type Value = f64;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("width")>,
    ) -> &Self::Value {
        &self.width
    }
}
impl HasFieldMut<Symbol!("width")> for Rectangle {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("width")>,
    ) -> &mut Self::Value {
        &mut self.width
    }
}
impl DelegateComponent<AreaCalculatorComponent> for Rectangle {
    type Delegate = RectangleArea;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<AreaCalculatorComponent, __Context__, __Params__> for Rectangle
where
    RectangleArea: IsProviderFor<AreaCalculatorComponent, __Context__, __Params__>,
{}
trait __CheckRectangle<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckRectangle<AreaCalculatorComponent, ()> for Rectangle {}
fn after() -> u32 {
    "not a number"
}
fn main() {
    let _ = after();
}
