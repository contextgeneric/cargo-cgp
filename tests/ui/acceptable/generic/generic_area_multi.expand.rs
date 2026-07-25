#![feature(prelude_import)]
//! Usability: the wiring-note and header rewrites over a component with *three* generic
//! parameters, so the parameters reach `CanUseComponent`/`IsProviderFor` grouped in a tuple.
//!
//! `CanCalculateArea<Scale, Offset, Unit>` has three type parameters, so the check is
//! `Rectangle: CanUseComponent<AreaCalculatorComponent, (u32, u64, bool)>` — the parameters
//! arrive as one tuple argument. The provider still needs a `width` field the `Rectangle`
//! lacks, so the failure surfaces through the same chain as `generic_area`, but now the
//! header must *unwrap* the tuple to name the trait as written (`CanCalculateArea<u32, u64,
//! bool>`). This fixture is the regression guard for that multi-parameter unwrapping.
//!
//! Exposes issues in cgp-knowledge-base/cargo-cgp/issues/usability.md. CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait CanCalculateArea<Scale, Offset, Unit> {
    fn area(&self) -> f64;
}
impl<__Context__, Scale, Offset, Unit> CanCalculateArea<Scale, Offset, Unit>
for __Context__
where
    __Context__: AreaCalculator<__Context__, Scale, Offset, Unit>,
{
    fn area(&self) -> f64 {
        __Context__::area(self)
    }
}
pub trait AreaCalculator<
    __Context__,
    Scale,
    Offset,
    Unit,
>: IsProviderFor<AreaCalculatorComponent, __Context__, (Scale, Offset, Unit)> {
    fn area(__context__: &__Context__) -> f64;
}
impl<
    __Provider__,
    __Context__,
    Scale,
    Offset,
    Unit,
> AreaCalculator<__Context__, Scale, Offset, Unit> for __Provider__
where
    __Provider__: DelegateComponent<AreaCalculatorComponent>
        + IsProviderFor<AreaCalculatorComponent, __Context__, (Scale, Offset, Unit)>,
    <__Provider__ as DelegateComponent<
        AreaCalculatorComponent,
    >>::Delegate: AreaCalculator<__Context__, Scale, Offset, Unit>,
{
    fn area(__context__: &__Context__) -> f64 {
        <__Provider__ as DelegateComponent<
            AreaCalculatorComponent,
        >>::Delegate::area(__context__)
    }
}
pub struct AreaCalculatorComponent;
impl<__Context__, Scale, Offset, Unit> AreaCalculator<__Context__, Scale, Offset, Unit>
for UseContext
where
    __Context__: CanCalculateArea<Scale, Offset, Unit>,
{
    fn area(__context__: &__Context__) -> f64 {
        __Context__::area(__context__)
    }
}
impl<
    __Context__,
    Scale,
    Offset,
    Unit,
> IsProviderFor<AreaCalculatorComponent, __Context__, (Scale, Offset, Unit)>
for UseContext
where
    __Context__: CanCalculateArea<Scale, Offset, Unit>,
{}
impl<
    __Context__,
    Scale,
    Offset,
    Unit,
    __Components__,
    __Path__,
> AreaCalculator<__Context__, Scale, Offset, Unit>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Scale.Offset.Unit)>,
    __Components__: DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Scale.Offset.Unit)>>::Output,
    >,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Scale.Offset.Unit)>>::Output,
    >>::Delegate: AreaCalculator<__Context__, Scale, Offset, Unit>,
{
    fn area(__context__: &__Context__) -> f64 {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Scale.Offset.Unit)>>::Output,
        >>::Delegate::area(__context__)
    }
}
impl<
    __Context__,
    Scale,
    Offset,
    Unit,
    __Components__,
    __Path__,
> IsProviderFor<AreaCalculatorComponent, __Context__, (Scale, Offset, Unit)>
for RedirectLookup<__Components__, __Path__>
where
    __Path__: ConcatPath<Path!(@Scale.Offset.Unit)>,
    __Components__: DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Scale.Offset.Unit)>>::Output,
    >,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Scale.Offset.Unit)>>::Output,
    >>::Delegate: IsProviderFor<
            AreaCalculatorComponent,
            __Context__,
            (Scale, Offset, Unit),
        > + AreaCalculator<__Context__, Scale, Offset, Unit>,
{}
pub trait HasWidth {
    fn width(&self) -> f64;
}
impl<__Context__> HasWidth for __Context__
where
    __Context__: HasField<Symbol!("width"), Value = f64>,
{
    fn width(&self) -> f64 {
        self.get_field(::core::marker::PhantomData::<Symbol!("width")>).clone()
    }
}
impl<__Context__> AreaCalculator<__Context__, u32, u64, bool> for RectangleArea
where
    __Context__: HasWidth,
{
    fn area(__context__: &__Context__) -> f64 {
        __context__.width()
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, (u32, u64, bool)>
for RectangleArea
where
    __Context__: HasWidth,
{}
pub struct RectangleArea;
pub struct Rectangle {
    pub height: f64,
}
impl HasField<Symbol!("height")> for Rectangle {
    type Value = f64;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("height")>,
    ) -> &Self::Value {
        &self.height
    }
}
impl HasFieldMut<Symbol!("height")> for Rectangle {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("height")>,
    ) -> &mut Self::Value {
        &mut self.height
    }
}
impl DelegateComponent<AreaCalculatorComponent> for Rectangle {
    type Delegate = RectangleArea;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<AreaCalculatorComponent, __Context__, __Params__> for Rectangle
where
    RectangleArea: IsProviderFor<AreaCalculatorComponent, __Context__, __Params__>,
{}
trait __CheckRectangle<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckRectangle<AreaCalculatorComponent, (u32, u64, bool)> for Rectangle {}
fn main() {}
