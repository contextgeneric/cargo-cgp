#![feature(prelude_import)]
//! Acceptable failure: the ordinary-trait-bound dependency reached through *impl
//! generics* in `delegate_components!`, rather than a concrete context.
//!
//! A generic context `<T> Wrapper<T>` wires its abstract `Scalar` type to the impl
//! generic `T` (`ScalarTypeProviderComponent: UseType<T>`), and `CompareScalars`
//! needs `Scalar: Eq` — i.e. `T: Eq`. The generic wiring is accepted unconditionally;
//! the bound only bites at a concrete instantiation. Checking `Wrapper<f64>` surfaces
//! `f64: Eq` unsatisfied through `IsProviderFor<ScalarEqualityComponent, Wrapper<f64>>`,
//! exactly as the concrete-context case does — showing the ordinary-trait-bound class
//! arises anywhere impl generics carry a bound, including a generic
//! `delegate_components!` table checked at one instantiation.
//!
//! See docs/errors/checks/ordinary-trait-bound.md.
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
pub trait CanCompareScalars: HasScalarType {
    fn scalars_equal(
        &self,
        a: &<Self as HasScalarType>::Scalar,
        b: &<Self as HasScalarType>::Scalar,
    ) -> bool;
}
impl<__Context__> CanCompareScalars for __Context__
where
    __Context__: HasScalarType,
    __Context__: ScalarEquality<__Context__>,
{
    fn scalars_equal(
        &self,
        a: &<Self as HasScalarType>::Scalar,
        b: &<Self as HasScalarType>::Scalar,
    ) -> bool {
        __Context__::scalars_equal(self, a, b)
    }
}
pub trait ScalarEquality<
    __Context__,
>: IsProviderFor<ScalarEqualityComponent, __Context__, ()>
where
    __Context__: HasScalarType,
{
    fn scalars_equal(
        __context__: &__Context__,
        a: &<__Context__ as HasScalarType>::Scalar,
        b: &<__Context__ as HasScalarType>::Scalar,
    ) -> bool;
}
impl<__Provider__, __Context__> ScalarEquality<__Context__> for __Provider__
where
    __Context__: HasScalarType,
    __Provider__: DelegateComponent<ScalarEqualityComponent>
        + IsProviderFor<ScalarEqualityComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        ScalarEqualityComponent,
    >>::Delegate: ScalarEquality<__Context__>,
{
    fn scalars_equal(
        __context__: &__Context__,
        a: &<__Context__ as HasScalarType>::Scalar,
        b: &<__Context__ as HasScalarType>::Scalar,
    ) -> bool {
        <__Provider__ as DelegateComponent<
            ScalarEqualityComponent,
        >>::Delegate::scalars_equal(__context__, a, b)
    }
}
pub struct ScalarEqualityComponent;
impl<__Context__> ScalarEquality<__Context__> for UseContext
where
    __Context__: HasScalarType,
    __Context__: CanCompareScalars,
{
    fn scalars_equal(
        __context__: &__Context__,
        a: &<__Context__ as HasScalarType>::Scalar,
        b: &<__Context__ as HasScalarType>::Scalar,
    ) -> bool {
        __Context__::scalars_equal(__context__, a, b)
    }
}
impl<__Context__> IsProviderFor<ScalarEqualityComponent, __Context__, ()> for UseContext
where
    __Context__: HasScalarType,
    __Context__: CanCompareScalars,
{}
impl<__Context__, __Components__, __Path__> ScalarEquality<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasScalarType,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: ScalarEquality<__Context__>,
{
    fn scalars_equal(
        __context__: &__Context__,
        a: &<__Context__ as HasScalarType>::Scalar,
        b: &<__Context__ as HasScalarType>::Scalar,
    ) -> bool {
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate::scalars_equal(__context__, a, b)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<ScalarEqualityComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasScalarType,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<ScalarEqualityComponent, __Context__, ()>
        + ScalarEquality<__Context__>,
{}
impl<__Context__> ScalarEquality<__Context__> for CompareScalars
where
    <__Context__ as HasScalarType>::Scalar: Eq,
    __Context__: HasScalarType,
{
    fn scalars_equal(
        __context__: &__Context__,
        a: &<__Context__ as HasScalarType>::Scalar,
        b: &<__Context__ as HasScalarType>::Scalar,
    ) -> bool {
        a == b
    }
}
impl<__Context__> IsProviderFor<ScalarEqualityComponent, __Context__, ()>
for CompareScalars
where
    <__Context__ as HasScalarType>::Scalar: Eq,
    __Context__: HasScalarType,
{}
pub struct CompareScalars;
pub struct Wrapper<T>(pub T);
impl<T> DelegateComponent<ScalarTypeProviderComponent> for Wrapper<T> {
    type Delegate = UseType<T>;
}
impl<
    T,
    __Context__,
    __Params__,
> IsProviderFor<ScalarTypeProviderComponent, __Context__, __Params__> for Wrapper<T>
where
    UseType<T>: IsProviderFor<ScalarTypeProviderComponent, __Context__, __Params__>,
{}
impl<T> DelegateComponent<ScalarEqualityComponent> for Wrapper<T> {
    type Delegate = CompareScalars;
}
impl<
    T,
    __Context__,
    __Params__,
> IsProviderFor<ScalarEqualityComponent, __Context__, __Params__> for Wrapper<T>
where
    CompareScalars: IsProviderFor<ScalarEqualityComponent, __Context__, __Params__>,
{}
trait __CheckWrapper<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckWrapper<ScalarEqualityComponent, ()> for Wrapper<f64> {}
fn main() {}
