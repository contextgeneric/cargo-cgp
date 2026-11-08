#![feature(prelude_import)]
//! `#[use_type]` imports an associated type name the owning trait does not declare, so the
//! substituted `<Self as Trait>::WrongName` path names an associated type that does not exist and
//! the compiler rejects it.
//!
//! `HasScalarType` declares `Scalar`, but the import names `Scalr` (a typo), so the bare `Scalr` in
//! the signature is rewritten to `<Self as HasScalarType>::Scalr`. CGP cannot know the trait's
//! associated types at expansion time, since it performs a textual rewrite, so it lowers the name
//! faithfully and defers to the compiler, which reports `E0576` "cannot find associated type
//! `Scalr`". The substitution preserves the span of the identifier the user wrote, so the caret
//! lands on the `Scalr` in the signature, not on the macro attribute, and this fixture also guards
//! that span behavior. rustc's suggestion respells the signature only, which leaves the attribute's
//! `Scalr` in place and fails with `E0425` on `Scalar`, so the tool drops it and keeps rustc's
//! header, with a `help` naming the rename in the import and in each use of the alias.
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
///The trait for the `get_scalar` operation.
pub trait GetScalar: HasScalarType {
    ///The `get_scalar` operation.
    fn get_scalar(&self) -> <Self as HasScalarType>::Scalr;
}
impl<__Context__> GetScalar for __Context__
where
    Self: HasScalarType,
{
    fn get_scalar(&self) -> <Self as HasScalarType>::Scalr {
        ::core::panicking::todo_default_msg()
    }
}
fn main() {}
