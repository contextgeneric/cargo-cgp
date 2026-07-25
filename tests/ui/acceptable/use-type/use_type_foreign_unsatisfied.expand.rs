#![feature(prelude_import)]
//! Acceptable failure: a foreign `#[use_type(HasScalarType.Scalar in Types)]` import
//! adds `Types: HasScalarType` to the generated trait, so naming the component for
//! a `Types` that does not implement `HasScalarType` is rejected by the compiler.
//!
//! This is the constraint that used to be *silently dropped* — before the trait
//! carried the foreign bound, `NoScalar` would have slipped through here and only
//! failed much later (or not at all, if the abstract type went unused). CGP is now
//! working as designed: it emits the bound and defers the actual check to `rustc`,
//! which reports the missing `NoScalar: HasScalarType` at the use site.
//!
//! See docs/reference/attributes/use_type.md and docs/errors/checks/check-trait-failure.md.
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
pub trait CanCalculateArea<Types>
where
    Types: HasScalarType,
{
    fn area(&self) -> <Types as HasScalarType>::Scalar;
}
impl<__Context__, Types> CanCalculateArea<Types> for __Context__
where
    Types: HasScalarType,
    __Context__: AreaCalculator<__Context__, Types>,
{
    fn area(&self) -> <Types as HasScalarType>::Scalar {
        __Context__::area(self)
    }
}
pub trait AreaCalculator<
    __Context__,
    Types,
>: IsProviderFor<AreaCalculatorComponent, __Context__, (Types)>
where
    Types: HasScalarType,
{
    fn area(__context__: &__Context__) -> <Types as HasScalarType>::Scalar;
}
impl<__Provider__, __Context__, Types> AreaCalculator<__Context__, Types>
for __Provider__
where
    Types: HasScalarType,
    __Provider__: DelegateComponent<AreaCalculatorComponent>
        + IsProviderFor<AreaCalculatorComponent, __Context__, (Types)>,
    <__Provider__ as DelegateComponent<
        AreaCalculatorComponent,
    >>::Delegate: AreaCalculator<__Context__, Types>,
{
    fn area(__context__: &__Context__) -> <Types as HasScalarType>::Scalar {
        <__Provider__ as DelegateComponent<
            AreaCalculatorComponent,
        >>::Delegate::area(__context__)
    }
}
pub struct AreaCalculatorComponent;
impl<__Context__, Types> AreaCalculator<__Context__, Types> for UseContext
where
    Types: HasScalarType,
    __Context__: CanCalculateArea<Types>,
{
    fn area(__context__: &__Context__) -> <Types as HasScalarType>::Scalar {
        __Context__::area(__context__)
    }
}
impl<__Context__, Types> IsProviderFor<AreaCalculatorComponent, __Context__, (Types)>
for UseContext
where
    Types: HasScalarType,
    __Context__: CanCalculateArea<Types>,
{}
impl<__Context__, Types, __Components__, __Path__> AreaCalculator<__Context__, Types>
for RedirectLookup<__Components__, __Path__>
where
    Types: HasScalarType,
    __Path__: ConcatPath<Path!(@Types)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Types)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Types)>>::Output,
    >>::Delegate: AreaCalculator<__Context__, Types>,
{
    fn area(__context__: &__Context__) -> <Types as HasScalarType>::Scalar {
        <__Components__ as DelegateComponent<
            <__Path__ as ConcatPath<Path!(@Types)>>::Output,
        >>::Delegate::area(__context__)
    }
}
impl<
    __Context__,
    Types,
    __Components__,
    __Path__,
> IsProviderFor<AreaCalculatorComponent, __Context__, (Types)>
for RedirectLookup<__Components__, __Path__>
where
    Types: HasScalarType,
    __Path__: ConcatPath<Path!(@Types)>,
    __Components__: DelegateComponent<<__Path__ as ConcatPath<Path!(@Types)>>::Output>,
    <__Components__ as DelegateComponent<
        <__Path__ as ConcatPath<Path!(@Types)>>::Output,
    >>::Delegate: IsProviderFor<AreaCalculatorComponent, __Context__, (Types)>
        + AreaCalculator<__Context__, Types>,
{}
pub struct NoScalar;
pub trait CheckMissingScalar: CanCalculateArea<NoScalar> {}
fn main() {}
