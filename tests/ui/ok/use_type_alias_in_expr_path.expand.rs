#![feature(prelude_import)]
//! Clean compile: a `#[use_type]` alias written bare where it *qualifies* an
//! expression path.
//!
//! `#[use_type]` rewrites each bare occurrence of an imported alias into its
//! qualified `<Self as Trait>::Assoc` form, and that includes an alias heading an
//! expression path: `Transaction::begin_from(database)` becomes
//! `<<Self as HasTransactionType>::Transaction>::begin_from(database)`. The same
//! fixture reads the alias in three further positions — a `where` predicate, an
//! `#[implicit]` annotation, and the return type — so every position is pinned
//! together in one program.
//!
//! This is a regression pin. The substitution once ran over type nodes only, so an
//! alias qualifying an expression path was passed through untouched and the body
//! failed to resolve with `E0433` — an inconsistency rather than a boundary, since a
//! `let` annotation in the same body *was* rewritten. What remains a boundary is
//! arity: a bare single-segment path in expression position names a value, which an
//! abstract type can never be, so it is left alone. Both halves are pinned at the
//! macro level by `use_type_fn_expr_path` in `cgp`'s `abstract_types` tests.
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
pub trait CanBeginFrom<Db> {
    fn begin_from(pool: &Pool<Db>) -> Self;
}
impl CanBeginFrom<Postgres> for Tx<Postgres> {
    fn begin_from(_pool: &Pool<Postgres>) -> Self {
        Tx(core::marker::PhantomData)
    }
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
    <Self as HasTransactionType>::Transaction: CanBeginFrom<<Self as HasDbType>::Db>,
    Self: HasField<Symbol!("database"), Value = Pool<<Self as HasDbType>::Db>>,
    Self: HasDbType,
    Self: HasTransactionType,
{
    fn begin_transaction(&self) -> <Self as HasTransactionType>::Transaction {
        let database: &Pool<<Self as HasDbType>::Db> = self
            .get_field(::core::marker::PhantomData::<Symbol!("database")>);
        <<Self as HasTransactionType>::Transaction>::begin_from(database)
    }
}
fn main() {}
