#![feature(prelude_import)]
//! Acceptable failure: a provider's impl-side dependency is an *ordinary Rust
//! trait bound* — a standard trait (`Eq`), not a CGP capability — on an abstract
//! type, and the concrete type the context wires for that abstract type does not
//! implement it.
//!
//! `CompareScalars` requires `Scalar: Eq` (rewritten by `#[use_type]` to
//! `<Self as HasScalarType>::Scalar: Eq`). `App` wires its `Scalar` type to `f64`,
//! which is `PartialEq` but not `Eq`, so the dependency is unmet. The wiring is
//! accepted lazily; forcing it through `check_components!` surfaces the failure via
//! `IsProviderFor` as `E0277` — but unlike a missing `HasField` (whose leaf sits in
//! a `help:` note under a `CanUseComponent` primary), the *primary* error names the
//! ordinary bound on the concrete type directly (`f64: Eq` is not satisfied), the
//! `help:` lists the standard types that *do* implement `Eq`, and the `IsProviderFor`
//! note points at the `Scalar: Eq` bound as "introduced here". The fix is to satisfy
//! the ordinary trait (wire an `Eq` type such as an integer, or derive/impl `Eq`),
//! not to wire a component or add a field.
//!
//! CGP lowers the bound faithfully and cannot see the wired type violates it, so it
//! defers to the compiler. This is the same lazy-wiring mechanism as a CGP-capability
//! dependency; only the *kind of leaf* (an ordinary trait) and the fix differ.
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
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<ScalarEqualityComponent, ()> for App {}
fn main() {}
