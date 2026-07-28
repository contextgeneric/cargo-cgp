#![feature(prelude_import)]
//! Acceptable failure: a field whose required type is computed from an abstract type
//! the *same context* wires, so the mismatch is between two of its own wiring
//! decisions.
//!
//! `CountPooledRows` reads a `database` field typed `&Pool<Db>`, where `Db` is the
//! abstract type imported with `#[use_type(HasDbType.Db)]`. The field bound the macro
//! emits is therefore `HasField<Symbol!("database"), Value = Pool<Self::Db>>` — the
//! required type is not a constant but a projection through the context's own wiring.
//! `App` wires `Db` to `Postgres` while declaring a `Pool<Sqlite>` field, so the two
//! disagree and the `HasField` projection fails as an `E0271`.
//!
//! This is what an abstract type buys over the `#[impl_generics]` form that merely
//! *infers* the type from the field: under inference the two could never disagree,
//! because there was only ever one of them. Naming the type makes the pairing a
//! stated claim, and a stated claim is checkable — the check reports it at the wiring
//! entry rather than leaving a reader to notice that the pool is the wrong engine.
//!
//! The sibling with a *concrete* required type is
//! [`field_type_mismatch`](field_type_mismatch.rs); the one with no field involved at
//! all, where a provider pins an abstract type the context wires differently, is
//! [`types/abstract_type_mismatch`](../types/abstract_type_mismatch.rs).
//!
//! See cgp-knowledge-base/cgp/errors/checks/check-trait-failure.md and
//! cgp-knowledge-base/cgp/guides/naming-a-type-dependency.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::prelude::*;
pub trait Database: Sized {
    type Row;
}
pub struct Pool<Db>(pub core::marker::PhantomData<Db>);
pub struct Postgres;
impl Database for Postgres {
    type Row = String;
}
pub struct Sqlite;
impl Database for Sqlite {
    type Row = String;
}
pub trait HasDbType {
    type Db: Database;
}
impl<__Context__> HasDbType for __Context__
where
    __Context__: DbTypeProvider<__Context__>,
{
    type Db = <__Context__ as DbTypeProvider<__Context__>>::Db;
}
pub trait DbTypeProvider<
    __Context__,
>: IsProviderFor<DbTypeProviderComponent, __Context__, ()> {
    type Db: Database;
}
impl<__Provider__, __Context__> DbTypeProvider<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<DbTypeProviderComponent>
        + IsProviderFor<DbTypeProviderComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        DbTypeProviderComponent,
    >>::Delegate: DbTypeProvider<__Context__>,
{
    type Db = <<__Provider__ as DelegateComponent<
        DbTypeProviderComponent,
    >>::Delegate as DbTypeProvider<__Context__>>::Db;
}
pub struct DbTypeProviderComponent;
impl<__Context__> DbTypeProvider<__Context__> for UseContext
where
    __Context__: HasDbType,
{
    type Db = <__Context__ as HasDbType>::Db;
}
impl<__Context__> IsProviderFor<DbTypeProviderComponent, __Context__, ()> for UseContext
where
    __Context__: HasDbType,
{}
impl<__Context__, __Components__, __Path__> DbTypeProvider<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: DbTypeProvider<__Context__>,
{
    type Db = <<__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate as DbTypeProvider<__Context__>>::Db;
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<DbTypeProviderComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<DbTypeProviderComponent, __Context__, ()>
        + DbTypeProvider<__Context__>,
{}
impl<Db, __Context__> DbTypeProvider<__Context__> for UseType<Db>
where
    Db: Database,
{
    type Db = Db;
}
impl<Db, __Context__> IsProviderFor<DbTypeProviderComponent, __Context__, ()>
for UseType<Db>
where
    Db: Database,
{}
impl<__Provider__, Db, __Context__> DbTypeProvider<__Context__>
for WithProvider<__Provider__>
where
    Db: Database,
    __Provider__: TypeProvider<__Context__, DbTypeProviderComponent, Type = Db>,
{
    type Db = Db;
}
impl<
    __Provider__,
    Db,
    __Context__,
> IsProviderFor<DbTypeProviderComponent, __Context__, ()> for WithProvider<__Provider__>
where
    Db: Database,
    __Provider__: TypeProvider<__Context__, DbTypeProviderComponent, Type = Db>,
{}
pub trait CanCountRows: HasErrorType {
    fn count_rows(&self) -> Result<u64, <Self as HasErrorType>::Error>;
}
impl<__Context__> CanCountRows for __Context__
where
    __Context__: HasErrorType,
    __Context__: RowCounter<__Context__>,
{
    fn count_rows(&self) -> Result<u64, <Self as HasErrorType>::Error> {
        __Context__::count_rows(self)
    }
}
pub trait RowCounter<__Context__>: IsProviderFor<RowCounterComponent, __Context__, ()>
where
    __Context__: HasErrorType,
{
    fn count_rows(
        __context__: &__Context__,
    ) -> Result<u64, <__Context__ as HasErrorType>::Error>;
}
impl<__Provider__, __Context__> RowCounter<__Context__> for __Provider__
where
    __Context__: HasErrorType,
    __Provider__: DelegateComponent<RowCounterComponent>
        + IsProviderFor<RowCounterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        RowCounterComponent,
    >>::Delegate: RowCounter<__Context__>,
{
    fn count_rows(
        __context__: &__Context__,
    ) -> Result<u64, <__Context__ as HasErrorType>::Error> {
        <__Provider__ as DelegateComponent<
            RowCounterComponent,
        >>::Delegate::count_rows(__context__)
    }
}
pub struct RowCounterComponent;
impl<__Context__> RowCounter<__Context__> for UseContext
where
    __Context__: HasErrorType,
    __Context__: CanCountRows,
{
    fn count_rows(
        __context__: &__Context__,
    ) -> Result<u64, <__Context__ as HasErrorType>::Error> {
        __Context__::count_rows(__context__)
    }
}
impl<__Context__> IsProviderFor<RowCounterComponent, __Context__, ()> for UseContext
where
    __Context__: HasErrorType,
    __Context__: CanCountRows,
{}
impl<__Context__, __Components__, __Path__> RowCounter<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasErrorType,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: RowCounter<__Context__>,
{
    fn count_rows(
        __context__: &__Context__,
    ) -> Result<u64, <__Context__ as HasErrorType>::Error> {
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate::count_rows(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<RowCounterComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasErrorType,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<RowCounterComponent, __Context__, ()>
        + RowCounter<__Context__>,
{}
impl<__Context__> RowCounter<__Context__> for CountPooledRows
where
    __Context__: HasField<
        Symbol!("database"),
        Value = Pool<<__Context__ as HasDbType>::Db>,
    >,
    __Context__: HasDbType,
    __Context__: HasErrorType,
{
    fn count_rows(
        __context__: &__Context__,
    ) -> Result<u64, <__Context__ as HasErrorType>::Error> {
        let database: &Pool<<__Context__ as HasDbType>::Db> = __context__
            .get_field(::core::marker::PhantomData::<Symbol!("database")>);
        let _ = database;
        Ok(0)
    }
}
impl<__Context__> IsProviderFor<RowCounterComponent, __Context__, ()> for CountPooledRows
where
    __Context__: HasField<
        Symbol!("database"),
        Value = Pool<<__Context__ as HasDbType>::Db>,
    >,
    __Context__: HasDbType,
    __Context__: HasErrorType,
{}
pub struct CountPooledRows;
pub struct App {
    pub database: Pool<Sqlite>,
}
impl HasField<Symbol!("database")> for App {
    type Value = Pool<Sqlite>;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("database")>,
    ) -> &Self::Value {
        &self.database
    }
}
impl HasFieldMut<Symbol!("database")> for App {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("database")>,
    ) -> &mut Self::Value {
        &mut self.database
    }
}
impl DelegateComponent<ErrorTypeProviderComponent> for App {
    type Delegate = UseType<String>;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__> for App
where
    UseType<String>: IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__>,
{}
impl DelegateComponent<DbTypeProviderComponent> for App {
    type Delegate = UseType<Postgres>;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<DbTypeProviderComponent, __Context__, __Params__> for App
where
    UseType<Postgres>: IsProviderFor<DbTypeProviderComponent, __Context__, __Params__>,
{}
impl DelegateComponent<RowCounterComponent> for App {
    type Delegate = CountPooledRows;
}
impl<__Context__, __Params__> IsProviderFor<RowCounterComponent, __Context__, __Params__>
for App
where
    CountPooledRows: IsProviderFor<RowCounterComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<RowCounterComponent, ()> for App {}
fn main() {}
