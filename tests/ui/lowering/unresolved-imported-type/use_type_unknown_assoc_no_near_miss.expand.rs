#![feature(prelude_import)]
//! `#[use_type]` imports an associated type name the owning trait does not declare, and nothing it
//! declares is spelled close to it.
//!
//! `HasScalarType` declares only `Scalar`, but the import names `Weight`, so the bare `Weight` in
//! the signature is rewritten to `<Self as HasScalarType>::Weight` and the compiler reports `E0576`
//! at it. rustc suggests the trait's one associated type in the signature, which leaves the
//! import's `Weight` in place. That type is the one name an import of the trait can give, so the
//! tool drops the suggestion and names the rename in the import instead.
//!
//! CGP error class: unresolved imported abstract type (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/unresolved-imported-type.md).
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
///Component marker for [`HasScalarType`].
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
///The trait for the `get_weight` operation.
pub trait GetWeight: HasScalarType {
    ///The `get_weight` operation.
    fn get_weight(&self) -> <Self as HasScalarType>::Weight;
}
impl<__Context__> GetWeight for __Context__
where
    Self: HasScalarType,
{
    fn get_weight(&self) -> <Self as HasScalarType>::Weight {
        ::core::panicking::todo_default_msg()
    }
}
fn main() {}
