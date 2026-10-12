#![feature(prelude_import)]
//! A wiring key guessed from the trait name, `HasScalarTypeComponent`.
//!
//! The component of `HasScalarType` is `ScalarTypeProviderComponent`, so the guessed key is an
//! `E0425` "cannot find type", reported twice. The tool reshapes it into one `[CGP-E042]` whose
//! `help` names the real component.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).
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
pub struct App;
impl DelegateComponent<HasScalarTypeComponent> for App {
    type Delegate = UseType<f64>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<HasScalarTypeComponent, __Context__, __Params__> for App
where
    UseType<f64>: IsProviderFor<HasScalarTypeComponent, __Context__, __Params__>,
{}
fn main() {}
