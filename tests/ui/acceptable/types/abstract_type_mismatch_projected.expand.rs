#![feature(prelude_import)]
//! Acceptable failure: an abstract type a provider pins to a *projection* through another
//! abstract type, which the context binds to something else.
//!
//! The sibling [`abstract_type_mismatch`](abstract_type_mismatch.rs) pins the concrete
//! case, where a provider pins `Scalar` to `f64` and the context wires `UseType<u32>`.
//! Here the pin's right-hand side is `Tx<Db>` — a type projecting through the abstract
//! `Db` the same context wires — so the requirement the provider states is
//! `Tx<<App as HasDbType>::Db>` rather than a constant, and the two things in conflict
//! are both the context's own wiring decisions.
//!
//! This shape only became reachable once a pin grounded an alias *nested inside* its
//! right-hand side, so the fixture is the mismatch counterpart of
//! [`ok/use_type_pin_nested_alias`](../../ok/use_type_pin_nested_alias.rs). It pins the
//! dual rendering on the `[CGP-E017]` header and the `[CGP-E112]` leaf: the required type
//! appears as the projection the provider wrote, followed by what it reduces to, on the
//! same rule the field leaf uses — the projection names the wiring to change, the
//! reduction is what the reader compares against. A pin to a concrete type normalizes to
//! itself and gets no parenthetical, which is what the sibling fixture pins.
//!
//! See cgp-knowledge-base/cgp/errors/checks/check-trait-failure.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub trait Database: Sized {
    type Row;
}
pub struct Tx<Db>(pub core::marker::PhantomData<Db>);
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
pub trait CanNameTransaction: HasTransactionType {
    fn name_transaction(
        &self,
        transaction: <Self as HasTransactionType>::Transaction,
    ) -> &'static str;
}
impl<__Context__> CanNameTransaction for __Context__
where
    __Context__: HasTransactionType,
    __Context__: TransactionNamer<__Context__>,
{
    fn name_transaction(
        &self,
        transaction: <Self as HasTransactionType>::Transaction,
    ) -> &'static str {
        __Context__::name_transaction(self, transaction)
    }
}
pub trait TransactionNamer<
    __Context__,
>: IsProviderFor<TransactionNamerComponent, __Context__, ()>
where
    __Context__: HasTransactionType,
{
    fn name_transaction(
        __context__: &__Context__,
        transaction: <__Context__ as HasTransactionType>::Transaction,
    ) -> &'static str;
}
impl<__Provider__, __Context__> TransactionNamer<__Context__> for __Provider__
where
    __Context__: HasTransactionType,
    __Provider__: DelegateComponent<TransactionNamerComponent>
        + IsProviderFor<TransactionNamerComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        TransactionNamerComponent,
    >>::Delegate: TransactionNamer<__Context__>,
{
    fn name_transaction(
        __context__: &__Context__,
        transaction: <__Context__ as HasTransactionType>::Transaction,
    ) -> &'static str {
        <__Provider__ as DelegateComponent<
            TransactionNamerComponent,
        >>::Delegate::name_transaction(__context__, transaction)
    }
}
pub struct TransactionNamerComponent;
impl<__Context__> TransactionNamer<__Context__> for UseContext
where
    __Context__: HasTransactionType,
    __Context__: CanNameTransaction,
{
    fn name_transaction(
        __context__: &__Context__,
        transaction: <__Context__ as HasTransactionType>::Transaction,
    ) -> &'static str {
        __Context__::name_transaction(__context__, transaction)
    }
}
impl<__Context__> IsProviderFor<TransactionNamerComponent, __Context__, ()>
for UseContext
where
    __Context__: HasTransactionType,
    __Context__: CanNameTransaction,
{}
impl<__Context__, __Components__, __Path__> TransactionNamer<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasTransactionType,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: TransactionNamer<__Context__>,
{
    fn name_transaction(
        __context__: &__Context__,
        transaction: <__Context__ as HasTransactionType>::Transaction,
    ) -> &'static str {
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate::name_transaction(__context__, transaction)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<TransactionNamerComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasTransactionType,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<TransactionNamerComponent, __Context__, ()>
        + TransactionNamer<__Context__>,
{}
impl<__Context__> TransactionNamer<__Context__> for NamePooledTransaction
where
    __Context__: HasDbType,
    __Context__: HasTransactionType<Transaction = Tx<<__Context__ as HasDbType>::Db>>,
{
    fn name_transaction(
        __context__: &__Context__,
        transaction: <__Context__ as HasTransactionType>::Transaction,
    ) -> &'static str {
        let _ = transaction;
        "pooled"
    }
}
impl<__Context__> IsProviderFor<TransactionNamerComponent, __Context__, ()>
for NamePooledTransaction
where
    __Context__: HasDbType,
    __Context__: HasTransactionType<Transaction = Tx<<__Context__ as HasDbType>::Db>>,
{}
pub struct NamePooledTransaction;
pub struct App;
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
    type Delegate = UseType<Tx<Sqlite>>;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<TransactionTypeProviderComponent, __Context__, __Params__> for App
where
    UseType<
        Tx<Sqlite>,
    >: IsProviderFor<TransactionTypeProviderComponent, __Context__, __Params__>,
{}
impl DelegateComponent<TransactionNamerComponent> for App {
    type Delegate = NamePooledTransaction;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<TransactionNamerComponent, __Context__, __Params__> for App
where
    NamePooledTransaction: IsProviderFor<
        TransactionNamerComponent,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<TransactionNamerComponent, ()> for App {}
fn main() {}
