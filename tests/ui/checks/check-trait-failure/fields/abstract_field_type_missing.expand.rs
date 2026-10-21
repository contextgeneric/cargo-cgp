#![feature(prelude_import)]
//! A provider reads a field whose required type goes through an abstract type, on a context that
//! lacks the field.
//!
//! `CountPooledRows` reads `#[implicit] database: &Pool<Db>` under `#[use_type(HasDbType.Db)]`, so
//! its requirement is `HasField<Symbol!("database"), Value = Pool<<App as HasDbType>::Db>>`, a type
//! that still holds a projection. `App` derives `HasField` but has no `database`, so the tool
//! reports the missing field and names the fix without a type
//! (``add the field `database` to `App` ``): a help names only a type with no parameter,
//! placeholder, or projection left in it.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
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
    pub name: String,
}
impl HasField<Symbol!("name")> for App {
    type Value = String;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("name")>,
    ) -> &Self::Value {
        &self.name
    }
}
impl HasFieldMut<Symbol!("name")> for App {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("name")>,
    ) -> &mut Self::Value {
        &mut self.name
    }
}
impl DelegateComponent<ErrorTypeProviderComponent> for App {
    type Delegate = UseType<String>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__> for App
where
    UseType<String>: IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__>,
{}
impl DelegateComponent<DbTypeProviderComponent> for App {
    type Delegate = UseType<Postgres>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<DbTypeProviderComponent, __Context__, __Params__> for App
where
    UseType<Postgres>: IsProviderFor<DbTypeProviderComponent, __Context__, __Params__>,
{}
impl DelegateComponent<RowCounterComponent> for App {
    type Delegate = CountPooledRows;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<RowCounterComponent, __Context__, __Params__> for App
where
    CountPooledRows: IsProviderFor<RowCounterComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<RowCounterComponent, ()> for App {}
fn main() {}
