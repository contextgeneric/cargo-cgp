#![feature(prelude_import)]
//! An **abstract type** the context binds to one concrete type while a provider it uses pins the
//! same abstract type to another. `HasScalarType` is an [abstract-type
//! component](https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/abstract-types.md):
//! generic code names `Scalar` without committing to a concrete type, and the context chooses one
//! by wiring `ScalarTypeProviderComponent` to `UseType<T>`. Here `Rectangle` wires it to
//! `UseType<u32>`, but `RectangleArea` pins it with the `#[use_type(HasScalarType.{Scalar = f64})]`
//! equality form, so the provider needs `Scalar = f64`.
//!
//! This is the abstract-type sibling of the `HasField` mismatch in
//! [`field_type_mismatch`](../field-types/field_type_mismatch.rs), and it fails the same way:
//! `Rectangle: HasScalarType` *is* implemented, so the trait bound holds and only the
//! associated-type projection `<Rectangle as HasScalarType>::Scalar == f64` fails — an `E0271`.
//!
//! The driver resolves it the same way too: the walk reaches the provider impl whose every
//! trait-clause dependency holds, finds the unmet projection it carries, normalizes
//! `<Rectangle as HasScalarType>::Scalar` to read the type the context actually supplies (`u32`),
//! and rewrites the main message into the `[CGP-E017]` abstract-type form over the dependency
//! chain. Because the trait is a `#[cgp_type]` component — recognized structurally, by its provider
//! carrying the `UseType` blanket — a `help` names the wiring entry to change. Where rustc's raw
//! output aims `expected this to be `f64`` at the `#[cgp_type]` attribute and never states the type
//! the context supplies at all, the reshaped message names both sides and the fix. The `E0271` Rust
//! code is kept.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait HasScalarType {
    type Scalar;
}
impl<__Context__> HasScalarType for __Context__
where
    __Context__: ScalarTypeProvider<__Context__>,
{
    type Scalar = <__Context__ as ScalarTypeProvider<__Context__>>::Scalar;
}
pub trait ScalarTypeProvider<
    __Context__,
>: IsProviderFor<ScalarTypeProviderComponent, __Context__, ()> {
    type Scalar;
}
impl<__Provider__, __Context__> ScalarTypeProvider<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<ScalarTypeProviderComponent>
        + IsProviderFor<ScalarTypeProviderComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        ScalarTypeProviderComponent,
    >>::Delegate: ScalarTypeProvider<__Context__>,
{
    type Scalar = <<__Provider__ as DelegateComponent<
        ScalarTypeProviderComponent,
    >>::Delegate as ScalarTypeProvider<__Context__>>::Scalar;
}
pub struct ScalarTypeProviderComponent;
impl<__Context__> ScalarTypeProvider<__Context__> for UseContext
where
    __Context__: HasScalarType,
{
    type Scalar = <__Context__ as HasScalarType>::Scalar;
}
impl<__Context__> IsProviderFor<ScalarTypeProviderComponent, __Context__, ()>
for UseContext
where
    __Context__: HasScalarType,
{}
impl<__Context__, __Components__, __Path__> ScalarTypeProvider<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: ScalarTypeProvider<__Context__>,
{
    type Scalar = <<__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate as ScalarTypeProvider<__Context__>>::Scalar;
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<ScalarTypeProviderComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<ScalarTypeProviderComponent, __Context__, ()>
        + ScalarTypeProvider<__Context__>,
{}
impl<Scalar, __Context__> ScalarTypeProvider<__Context__> for UseType<Scalar> {
    type Scalar = Scalar;
}
impl<Scalar, __Context__> IsProviderFor<ScalarTypeProviderComponent, __Context__, ()>
for UseType<Scalar> {}
impl<__Provider__, Scalar, __Context__> ScalarTypeProvider<__Context__>
for WithProvider<__Provider__>
where
    __Provider__: TypeProvider<__Context__, ScalarTypeProviderComponent, Type = Scalar>,
{
    type Scalar = Scalar;
}
impl<
    __Provider__,
    Scalar,
    __Context__,
> IsProviderFor<ScalarTypeProviderComponent, __Context__, ()>
for WithProvider<__Provider__>
where
    __Provider__: TypeProvider<__Context__, ScalarTypeProviderComponent, Type = Scalar>,
{}
pub trait CanCalculateArea: HasScalarType {
    fn area(&self) -> <Self as HasScalarType>::Scalar;
}
impl<__Context__> CanCalculateArea for __Context__
where
    __Context__: HasScalarType,
    __Context__: AreaCalculator<__Context__>,
{
    fn area(&self) -> <Self as HasScalarType>::Scalar {
        __Context__::area(self)
    }
}
pub trait AreaCalculator<
    __Context__,
>: IsProviderFor<AreaCalculatorComponent, __Context__, ()>
where
    __Context__: HasScalarType,
{
    fn area(__context__: &__Context__) -> <__Context__ as HasScalarType>::Scalar;
}
impl<__Provider__, __Context__> AreaCalculator<__Context__> for __Provider__
where
    __Context__: HasScalarType,
    __Provider__: DelegateComponent<AreaCalculatorComponent>
        + IsProviderFor<AreaCalculatorComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        AreaCalculatorComponent,
    >>::Delegate: AreaCalculator<__Context__>,
{
    fn area(__context__: &__Context__) -> <__Context__ as HasScalarType>::Scalar {
        <__Provider__ as DelegateComponent<
            AreaCalculatorComponent,
        >>::Delegate::area(__context__)
    }
}
pub struct AreaCalculatorComponent;
impl<__Context__> AreaCalculator<__Context__> for UseContext
where
    __Context__: HasScalarType,
    __Context__: CanCalculateArea,
{
    fn area(__context__: &__Context__) -> <__Context__ as HasScalarType>::Scalar {
        __Context__::area(__context__)
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, ()> for UseContext
where
    __Context__: HasScalarType,
    __Context__: CanCalculateArea,
{}
impl<__Context__, __Components__, __Path__> AreaCalculator<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasScalarType,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: AreaCalculator<__Context__>,
{
    fn area(__context__: &__Context__) -> <__Context__ as HasScalarType>::Scalar {
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
    __Context__: HasScalarType,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<AreaCalculatorComponent, __Context__, ()>
        + AreaCalculator<__Context__>,
{}
impl<__Context__> AreaCalculator<__Context__> for RectangleArea
where
    __Context__: HasField<Symbol!("width"), Value = f64>
        + HasField<Symbol!("height"), Value = f64>,
    __Context__: HasScalarType<Scalar = f64>,
{
    fn area(__context__: &__Context__) -> f64 {
        let width: f64 = __context__
            .get_field(::core::marker::PhantomData::<Symbol!("width")>)
            .clone();
        let height: f64 = __context__
            .get_field(::core::marker::PhantomData::<Symbol!("height")>)
            .clone();
        width * height
    }
}
impl<__Context__> IsProviderFor<AreaCalculatorComponent, __Context__, ()>
for RectangleArea
where
    __Context__: HasField<Symbol!("width"), Value = f64>
        + HasField<Symbol!("height"), Value = f64>,
    __Context__: HasScalarType<Scalar = f64>,
{}
pub struct RectangleArea;
pub struct Rectangle {
    pub width: f64,
    pub height: f64,
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
impl DelegateComponent<ScalarTypeProviderComponent> for Rectangle {
    type Delegate = UseType<u32>;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<ScalarTypeProviderComponent, __Context__, __Params__> for Rectangle
where
    UseType<u32>: IsProviderFor<ScalarTypeProviderComponent, __Context__, __Params__>,
{}
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
impl __CheckRectangle<AreaCalculatorComponent, ()> for Rectangle {}
fn main() {}
