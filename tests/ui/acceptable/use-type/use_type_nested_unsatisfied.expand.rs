#![feature(prelude_import)]
//! Acceptable failure: a *nested* foreign `#[use_type(HasTypes.Types, HasScalarType.Scalar in Types)]`
//! import adds the two-hop bound `<Self as HasTypes>::Types: HasScalarType` to the
//! generated trait, so a context whose `Types` associated type does not implement
//! `HasScalarType` is rejected — proof the transitively-grounded foreign bound is
//! enforced at depth, not just for a directly-named parameter.
//!
//! Before the foreign bound was carried onto the trait, this nested constraint was
//! silently dropped. CGP is now working as designed: it emits the grounded bound
//! and defers the check to `rustc`, which reports the missing `NoScalar: HasScalarType`
//! at the site that requires `App: GetScalar`.
//!
//! See docs/reference/attributes/use_type.md and docs/errors/checks/check-trait-failure.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait HasTypes {
    type Types;
}
impl<__Context__> HasTypes for __Context__
where
    __Context__: TypesTypeProvider<__Context__>,
{
    type Types = <__Context__ as TypesTypeProvider<__Context__>>::Types;
}
pub trait TypesTypeProvider<
    __Context__,
>: IsProviderFor<TypesTypeProviderComponent, __Context__, ()> {
    type Types;
}
impl<__Provider__, __Context__> TypesTypeProvider<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<TypesTypeProviderComponent>
        + IsProviderFor<TypesTypeProviderComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        TypesTypeProviderComponent,
    >>::Delegate: TypesTypeProvider<__Context__>,
{
    type Types = <<__Provider__ as DelegateComponent<
        TypesTypeProviderComponent,
    >>::Delegate as TypesTypeProvider<__Context__>>::Types;
}
pub struct TypesTypeProviderComponent;
impl<__Context__> TypesTypeProvider<__Context__> for UseContext
where
    __Context__: HasTypes,
{
    type Types = <__Context__ as HasTypes>::Types;
}
impl<__Context__> IsProviderFor<TypesTypeProviderComponent, __Context__, ()>
for UseContext
where
    __Context__: HasTypes,
{}
impl<__Context__, __Components__, __Path__> TypesTypeProvider<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: TypesTypeProvider<__Context__>,
{
    type Types = <<__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate as TypesTypeProvider<__Context__>>::Types;
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<TypesTypeProviderComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<TypesTypeProviderComponent, __Context__, ()>
        + TypesTypeProvider<__Context__>,
{}
impl<Types, __Context__> TypesTypeProvider<__Context__> for UseType<Types> {
    type Types = Types;
}
impl<Types, __Context__> IsProviderFor<TypesTypeProviderComponent, __Context__, ()>
for UseType<Types> {}
impl<__Provider__, Types, __Context__> TypesTypeProvider<__Context__>
for WithProvider<__Provider__>
where
    __Provider__: TypeProvider<__Context__, TypesTypeProviderComponent, Type = Types>,
{
    type Types = Types;
}
impl<
    __Provider__,
    Types,
    __Context__,
> IsProviderFor<TypesTypeProviderComponent, __Context__, ()>
for WithProvider<__Provider__>
where
    __Provider__: TypeProvider<__Context__, TypesTypeProviderComponent, Type = Types>,
{}
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
pub trait GetScalar: HasTypes
where
    <Self as HasTypes>::Types: HasScalarType,
{
    fn get_scalar(&self) -> <<Self as HasTypes>::Types as HasScalarType>::Scalar;
}
impl<__Context__> GetScalar for __Context__
where
    Self: HasTypes,
    <Self as HasTypes>::Types: HasScalarType,
{
    fn get_scalar(&self) -> <<Self as HasTypes>::Types as HasScalarType>::Scalar {
        ::core::panicking::panic("not yet implemented")
    }
}
pub struct NoScalar;
pub struct App;
impl HasTypes for App {
    type Types = NoScalar;
}
fn assert_app()
where
    App: GetScalar,
{}
fn main() {}
