#![feature(prelude_import)]
//! Acceptable failure: an abstract type is given the same name as the trait that
//! bounds it, so the bound resolves to the associated type rather than the trait.
//!
//! `#[cgp_type] trait HasDatabaseType { type Database: Database; }` reads as "the
//! abstract type `Database`, which must implement the `Database` trait" — but inside
//! the declaration the name `Database` is already taken by the associated type being
//! declared, so the bound position resolves to that rather than to the trait in
//! scope. The compiler rejects it with `E0404` "expected trait, found type parameter
//! `Database`", the associated type having become a type parameter in the
//! `#[cgp_type]` expansion.
//!
//! The failure is a plain name collision rather than anything CGP-specific, but it
//! is easy to walk into when naming an abstract type after the concrete trait it
//! abstracts over — which is why an abstract database type reads better as `Db`,
//! as in [`ok/abstract_db_transaction.rs`](../../ok/abstract_db_transaction.rs).
//!
//! See cgp-knowledge-base/cgp/reference/macros/cgp_type.md and
//! cgp-knowledge-base/cgp/errors/lowering/out-of-scope-generated-name.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait Database: Sized {
    type Row;
}
pub struct Postgres;
impl Database for Postgres {
    type Row = String;
}
pub trait HasDatabaseType {
    type Database: Database;
}
impl<__Context__> HasDatabaseType for __Context__
where
    __Context__: DatabaseTypeProvider<__Context__>,
{
    type Database = <__Context__ as DatabaseTypeProvider<__Context__>>::Database;
}
pub trait DatabaseTypeProvider<
    __Context__,
>: IsProviderFor<DatabaseTypeProviderComponent, __Context__, ()> {
    type Database: Database;
}
impl<__Provider__, __Context__> DatabaseTypeProvider<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<DatabaseTypeProviderComponent>
        + IsProviderFor<DatabaseTypeProviderComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        DatabaseTypeProviderComponent,
    >>::Delegate: DatabaseTypeProvider<__Context__>,
{
    type Database = <<__Provider__ as DelegateComponent<
        DatabaseTypeProviderComponent,
    >>::Delegate as DatabaseTypeProvider<__Context__>>::Database;
}
pub struct DatabaseTypeProviderComponent;
impl<__Context__> DatabaseTypeProvider<__Context__> for UseContext
where
    __Context__: HasDatabaseType,
{
    type Database = <__Context__ as HasDatabaseType>::Database;
}
impl<__Context__> IsProviderFor<DatabaseTypeProviderComponent, __Context__, ()>
for UseContext
where
    __Context__: HasDatabaseType,
{}
impl<__Context__, __Components__, __Path__> DatabaseTypeProvider<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: DatabaseTypeProvider<__Context__>,
{
    type Database = <<__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate as DatabaseTypeProvider<__Context__>>::Database;
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<DatabaseTypeProviderComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<DatabaseTypeProviderComponent, __Context__, ()>
        + DatabaseTypeProvider<__Context__>,
{}
impl<Database, __Context__> DatabaseTypeProvider<__Context__> for UseType<Database>
where
    Database: Database,
{
    type Database = Database;
}
impl<Database, __Context__> IsProviderFor<DatabaseTypeProviderComponent, __Context__, ()>
for UseType<Database>
where
    Database: Database,
{}
impl<__Provider__, Database, __Context__> DatabaseTypeProvider<__Context__>
for WithProvider<__Provider__>
where
    Database: Database,
    __Provider__: TypeProvider<
        __Context__,
        DatabaseTypeProviderComponent,
        Type = Database,
    >,
{
    type Database = Database;
}
impl<
    __Provider__,
    Database,
    __Context__,
> IsProviderFor<DatabaseTypeProviderComponent, __Context__, ()>
for WithProvider<__Provider__>
where
    Database: Database,
    __Provider__: TypeProvider<
        __Context__,
        DatabaseTypeProviderComponent,
        Type = Database,
    >,
{}
fn main() {}
