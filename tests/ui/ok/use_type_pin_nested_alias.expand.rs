#![feature(prelude_import)]
//! Clean compile: a `#[use_type]` equality pin whose right-hand side *contains* an
//! imported alias rather than *being* one.
//!
//! The pin form `#[use_type(Trait.{Assoc = Type})]` emits `Self: Trait<Assoc = Type>`,
//! and an imported alias is grounded wherever it occurs inside that right-hand side —
//! so `{Transaction = Tx<Db>}` emits
//! `Self: HasTransactionType<Transaction = Tx<<Self as HasDbType>::Db>>`, naming the
//! projection rather than a bare `Db` that would resolve to nothing.
//!
//! This fixture is a regression pin. The substitution once ran only when the whole
//! right-hand side *was* an alias (the `{Bar as Baz = Foo}` cross-spec form), so an
//! alias nested inside one was passed through untouched and the emitted bound failed
//! to resolve with `E0425`, its caret inside the attribute and nothing in the message
//! saying the name had been imported. The macro-level expansion is pinned by
//! `use_type_fn_equality_nested` in `cgp`'s `abstract_types` tests; this fixture is
//! the end-to-end confirmation that a real crate using the form compiles silently.
//!
//! See cgp-knowledge-base/cgp/reference/attributes/use_type.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait Database: Sized {
    type Row;
}
pub struct Pool<Db>(pub core::marker::PhantomData<Db>);
pub struct Tx<Db>(pub core::marker::PhantomData<Db>);
pub struct Postgres;
impl Database for Postgres {
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
pub trait HasTransactionType {
    type Transaction;
}
impl<__Context__> HasTransactionType for __Context__
where
    __Context__: TransactionTypeProvider<__Context__>,
{
    type Transaction = <__Context__ as TransactionTypeProvider<
        __Context__,
    >>::Transaction;
}
pub trait TransactionTypeProvider<
    __Context__,
>: IsProviderFor<TransactionTypeProviderComponent, __Context__, ()> {
    type Transaction;
}
impl<__Provider__, __Context__> TransactionTypeProvider<__Context__> for __Provider__
where
    __Provider__: DelegateComponent<TransactionTypeProviderComponent>
        + IsProviderFor<TransactionTypeProviderComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        TransactionTypeProviderComponent,
    >>::Delegate: TransactionTypeProvider<__Context__>,
{
    type Transaction = <<__Provider__ as DelegateComponent<
        TransactionTypeProviderComponent,
    >>::Delegate as TransactionTypeProvider<__Context__>>::Transaction;
}
pub struct TransactionTypeProviderComponent;
impl<__Context__> TransactionTypeProvider<__Context__> for UseContext
where
    __Context__: HasTransactionType,
{
    type Transaction = <__Context__ as HasTransactionType>::Transaction;
}
impl<__Context__> IsProviderFor<TransactionTypeProviderComponent, __Context__, ()>
for UseContext
where
    __Context__: HasTransactionType,
{}
impl<__Context__, __Components__, __Path__> TransactionTypeProvider<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: TransactionTypeProvider<__Context__>,
{
    type Transaction = <<__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate as TransactionTypeProvider<__Context__>>::Transaction;
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<TransactionTypeProviderComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<TransactionTypeProviderComponent, __Context__, ()>
        + TransactionTypeProvider<__Context__>,
{}
impl<Transaction, __Context__> TransactionTypeProvider<__Context__>
for UseType<Transaction> {
    type Transaction = Transaction;
}
impl<
    Transaction,
    __Context__,
> IsProviderFor<TransactionTypeProviderComponent, __Context__, ()>
for UseType<Transaction> {}
impl<__Provider__, Transaction, __Context__> TransactionTypeProvider<__Context__>
for WithProvider<__Provider__>
where
    __Provider__: TypeProvider<
        __Context__,
        TransactionTypeProviderComponent,
        Type = Transaction,
    >,
{
    type Transaction = Transaction;
}
impl<
    __Provider__,
    Transaction,
    __Context__,
> IsProviderFor<TransactionTypeProviderComponent, __Context__, ()>
for WithProvider<__Provider__>
where
    __Provider__: TypeProvider<
        __Context__,
        TransactionTypeProviderComponent,
        Type = Transaction,
    >,
{}
pub trait BeginTransaction: HasDbType + HasTransactionType {
    fn begin_transaction(&self) -> <Self as HasTransactionType>::Transaction;
}
impl<__Context__> BeginTransaction for __Context__
where
    Self: HasField<Symbol!("database"), Value = Pool<<Self as HasDbType>::Db>>,
    Self: HasDbType,
    Self: HasTransactionType<Transaction = Tx<<Self as HasDbType>::Db>>,
{
    fn begin_transaction(&self) -> <Self as HasTransactionType>::Transaction {
        let database: &Pool<<Self as HasDbType>::Db> = self
            .get_field(::core::marker::PhantomData::<Symbol!("database")>);
        let _ = database;
        ::core::panicking::panic("not yet implemented")
    }
}
pub struct App {
    pub database: Pool<Postgres>,
}
impl HasField<Symbol!("database")> for App {
    type Value = Pool<Postgres>;
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
impl DelegateComponent<TransactionTypeProviderComponent> for App {
    type Delegate = UseType<Tx<Postgres>>;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<TransactionTypeProviderComponent, __Context__, __Params__> for App
where
    UseType<
        Tx<Postgres>,
    >: IsProviderFor<TransactionTypeProviderComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<DbTypeProviderComponent, ()> for App {}
impl __CheckApp<TransactionTypeProviderComponent, ()> for App {}
fn assert_can_begin<Context>()
where
    Context: BeginTransaction,
{}
fn main() {
    assert_can_begin::<App>();
}
