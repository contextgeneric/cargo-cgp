#![feature(prelude_import)]
//! Acceptable failure: the same unmet *ordinary Rust trait bound* dependency as
//! check_components/ordinary_bound_unsatisfied.rs (`Scalar: Eq` with `f64` wired),
//! but exercised by calling the consumer method rather than a check — so the cause
//! is *hidden*.
//!
//! Calling `app.scalars_equal(..)` produces the `E0599` "method exists but its
//! trait bounds were not satisfied" shape: it names `App: CanCompareScalars` /
//! `App: ScalarEquality<App>`, misclassifies the method as an associated function
//! (the provider method has no `self` receiver), and suggests `App::scalars_equal()`
//! — but never mentions the unmet `f64: Eq`. This is byte-for-shape identical to the
//! HasName hidden case in delegate_components/missing_dependency.rs: the compiler's
//! method-probe heuristic drops the nested `where`-clause bound regardless of whether
//! that bound is a `HasField`, a CGP capability, or an ordinary trait. Promote it with
//! `check_components!` to surface the `f64: Eq` cause.
//!
//! See cgp-knowledge-base/cgp/errors/hidden/unsatisfied-dependency.md; the surfaced counterpart is
//! cgp-knowledge-base/cgp/errors/checks/ordinary-trait-bound.md.
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
pub struct App;
impl DelegateComponent<ScalarTypeProviderComponent> for App {
    type Delegate = UseType<f64>;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<ScalarTypeProviderComponent, __Context__, __Params__> for App
where
    UseType<f64>: IsProviderFor<ScalarTypeProviderComponent, __Context__, __Params__>,
{}
impl DelegateComponent<ScalarEqualityComponent> for App {
    type Delegate = CompareScalars;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<ScalarEqualityComponent, __Context__, __Params__> for App
where
    CompareScalars: IsProviderFor<ScalarEqualityComponent, __Context__, __Params__>,
{}
fn main() {
    let app = App;
    let _ = app.scalars_equal(&1.0, &2.0);
}
