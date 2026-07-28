#![feature(prelude_import)]
//! Clean compile: two abstract types the context supplies by wiring, related by an
//! ordinary trait bound, with the capability that composes them naming neither.
//!
//! This is the positive counterpart to the lowering failures around abstract database
//! types. `Db` and `Transaction` are both abstract, so both are nameable in the
//! components' public signatures — which is what the inferred `#[impl_generics]` form
//! cannot do, per
//! [`lowering/impl_generics_in_signature`](../acceptable/lowering/impl_generics_in_signature.rs).
//! The relationship between them ("a transaction of *this* database") rides in an
//! ordinary `where` predicate over both bare aliases. A `#[use_type]` equality pin is
//! the alternative where the relationship is an equality rather than a trait bound,
//! and it grounds a nested alias too — see
//! [`use_type_pin_nested_alias`](use_type_pin_nested_alias.rs).
//!
//! The payoff is `run_unit_of_work`: it composes both capabilities and mentions neither
//! type, where a threaded generic parameter would have put `Db` and its bound in this
//! signature and in every caller's. The provider's body calls through the fully-qualified
//! `<Transaction as CanBeginFrom<Db>>::begin_from(…)` form, which names the trait
//! explicitly; the shorter `Transaction::begin_from(…)` also resolves, and
//! [`use_type_alias_in_expr_path`](use_type_alias_in_expr_path.rs) pins that.
//!
//! See cgp-knowledge-base/cgp/guides/naming-a-type-dependency.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::error::ErrorTypeProviderComponent;
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
pub trait CanBeginTransaction: HasTransactionType + HasErrorType {
    fn begin_transaction(
        &self,
    ) -> Result<
        <Self as HasTransactionType>::Transaction,
        <Self as HasErrorType>::Error,
    >;
}
impl<__Context__> CanBeginTransaction for __Context__
where
    __Context__: HasTransactionType + HasErrorType,
    __Context__: TransactionStarter<__Context__>,
{
    fn begin_transaction(
        &self,
    ) -> Result<
        <Self as HasTransactionType>::Transaction,
        <Self as HasErrorType>::Error,
    > {
        __Context__::begin_transaction(self)
    }
}
pub trait TransactionStarter<
    __Context__,
>: IsProviderFor<TransactionStarterComponent, __Context__, ()>
where
    __Context__: HasTransactionType + HasErrorType,
{
    fn begin_transaction(
        __context__: &__Context__,
    ) -> Result<
        <__Context__ as HasTransactionType>::Transaction,
        <__Context__ as HasErrorType>::Error,
    >;
}
impl<__Provider__, __Context__> TransactionStarter<__Context__> for __Provider__
where
    __Context__: HasTransactionType + HasErrorType,
    __Provider__: DelegateComponent<TransactionStarterComponent>
        + IsProviderFor<TransactionStarterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        TransactionStarterComponent,
    >>::Delegate: TransactionStarter<__Context__>,
{
    fn begin_transaction(
        __context__: &__Context__,
    ) -> Result<
        <__Context__ as HasTransactionType>::Transaction,
        <__Context__ as HasErrorType>::Error,
    > {
        <__Provider__ as DelegateComponent<
            TransactionStarterComponent,
        >>::Delegate::begin_transaction(__context__)
    }
}
pub struct TransactionStarterComponent;
impl<__Context__> TransactionStarter<__Context__> for UseContext
where
    __Context__: HasTransactionType + HasErrorType,
    __Context__: CanBeginTransaction,
{
    fn begin_transaction(
        __context__: &__Context__,
    ) -> Result<
        <__Context__ as HasTransactionType>::Transaction,
        <__Context__ as HasErrorType>::Error,
    > {
        __Context__::begin_transaction(__context__)
    }
}
impl<__Context__> IsProviderFor<TransactionStarterComponent, __Context__, ()>
for UseContext
where
    __Context__: HasTransactionType + HasErrorType,
    __Context__: CanBeginTransaction,
{}
impl<__Context__, __Components__, __Path__> TransactionStarter<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasTransactionType + HasErrorType,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: TransactionStarter<__Context__>,
{
    fn begin_transaction(
        __context__: &__Context__,
    ) -> Result<
        <__Context__ as HasTransactionType>::Transaction,
        <__Context__ as HasErrorType>::Error,
    > {
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate::begin_transaction(__context__)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<TransactionStarterComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasTransactionType + HasErrorType,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<TransactionStarterComponent, __Context__, ()>
        + TransactionStarter<__Context__>,
{}
pub trait CanCommitTransaction: HasTransactionType + HasErrorType {
    fn commit_transaction(
        &self,
        transaction: <Self as HasTransactionType>::Transaction,
    ) -> Result<(), <Self as HasErrorType>::Error>;
}
impl<__Context__> CanCommitTransaction for __Context__
where
    __Context__: HasTransactionType + HasErrorType,
    __Context__: TransactionCommitter<__Context__>,
{
    fn commit_transaction(
        &self,
        transaction: <Self as HasTransactionType>::Transaction,
    ) -> Result<(), <Self as HasErrorType>::Error> {
        __Context__::commit_transaction(self, transaction)
    }
}
pub trait TransactionCommitter<
    __Context__,
>: IsProviderFor<TransactionCommitterComponent, __Context__, ()>
where
    __Context__: HasTransactionType + HasErrorType,
{
    fn commit_transaction(
        __context__: &__Context__,
        transaction: <__Context__ as HasTransactionType>::Transaction,
    ) -> Result<(), <__Context__ as HasErrorType>::Error>;
}
impl<__Provider__, __Context__> TransactionCommitter<__Context__> for __Provider__
where
    __Context__: HasTransactionType + HasErrorType,
    __Provider__: DelegateComponent<TransactionCommitterComponent>
        + IsProviderFor<TransactionCommitterComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<
        TransactionCommitterComponent,
    >>::Delegate: TransactionCommitter<__Context__>,
{
    fn commit_transaction(
        __context__: &__Context__,
        transaction: <__Context__ as HasTransactionType>::Transaction,
    ) -> Result<(), <__Context__ as HasErrorType>::Error> {
        <__Provider__ as DelegateComponent<
            TransactionCommitterComponent,
        >>::Delegate::commit_transaction(__context__, transaction)
    }
}
pub struct TransactionCommitterComponent;
impl<__Context__> TransactionCommitter<__Context__> for UseContext
where
    __Context__: HasTransactionType + HasErrorType,
    __Context__: CanCommitTransaction,
{
    fn commit_transaction(
        __context__: &__Context__,
        transaction: <__Context__ as HasTransactionType>::Transaction,
    ) -> Result<(), <__Context__ as HasErrorType>::Error> {
        __Context__::commit_transaction(__context__, transaction)
    }
}
impl<__Context__> IsProviderFor<TransactionCommitterComponent, __Context__, ()>
for UseContext
where
    __Context__: HasTransactionType + HasErrorType,
    __Context__: CanCommitTransaction,
{}
impl<__Context__, __Components__, __Path__> TransactionCommitter<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasTransactionType + HasErrorType,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: TransactionCommitter<__Context__>,
{
    fn commit_transaction(
        __context__: &__Context__,
        transaction: <__Context__ as HasTransactionType>::Transaction,
    ) -> Result<(), <__Context__ as HasErrorType>::Error> {
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate::commit_transaction(__context__, transaction)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<TransactionCommitterComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasTransactionType + HasErrorType,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<TransactionCommitterComponent, __Context__, ()>
        + TransactionCommitter<__Context__>,
{}
impl<__Context__> TransactionStarter<__Context__> for BeginPooledTransaction
where
    <__Context__ as HasTransactionType>::Transaction: CanBeginFrom<
        <__Context__ as HasDbType>::Db,
    >,
    __Context__: HasField<
        Symbol!("database"),
        Value = Pool<<__Context__ as HasDbType>::Db>,
    >,
    __Context__: HasDbType,
    __Context__: HasTransactionType,
    __Context__: HasErrorType,
{
    fn begin_transaction(
        __context__: &__Context__,
    ) -> Result<
        <__Context__ as HasTransactionType>::Transaction,
        <__Context__ as HasErrorType>::Error,
    > {
        let database: &Pool<<__Context__ as HasDbType>::Db> = __context__
            .get_field(::core::marker::PhantomData::<Symbol!("database")>);
        Ok(
            <<__Context__ as HasTransactionType>::Transaction as CanBeginFrom<
                <__Context__ as HasDbType>::Db,
            >>::begin_from(database),
        )
    }
}
impl<__Context__> IsProviderFor<TransactionStarterComponent, __Context__, ()>
for BeginPooledTransaction
where
    <__Context__ as HasTransactionType>::Transaction: CanBeginFrom<
        <__Context__ as HasDbType>::Db,
    >,
    __Context__: HasField<
        Symbol!("database"),
        Value = Pool<<__Context__ as HasDbType>::Db>,
    >,
    __Context__: HasDbType,
    __Context__: HasTransactionType,
    __Context__: HasErrorType,
{}
pub struct BeginPooledTransaction;
impl<__Context__> TransactionCommitter<__Context__> for CommitPooledTransaction
where
    __Context__: HasTransactionType,
    __Context__: HasErrorType,
{
    fn commit_transaction(
        __context__: &__Context__,
        transaction: <__Context__ as HasTransactionType>::Transaction,
    ) -> Result<(), <__Context__ as HasErrorType>::Error> {
        let _ = transaction;
        Ok(())
    }
}
impl<__Context__> IsProviderFor<TransactionCommitterComponent, __Context__, ()>
for CommitPooledTransaction
where
    __Context__: HasTransactionType,
    __Context__: HasErrorType,
{}
pub struct CommitPooledTransaction;
pub trait RunUnitOfWork: HasErrorType {
    fn run_unit_of_work(&self) -> Result<(), <Self as HasErrorType>::Error>;
}
impl<__Context__> RunUnitOfWork for __Context__
where
    Self: CanBeginTransaction + CanCommitTransaction,
    Self: HasErrorType,
{
    fn run_unit_of_work(&self) -> Result<(), <Self as HasErrorType>::Error> {
        let transaction = self.begin_transaction()?;
        self.commit_transaction(transaction)?;
        Ok(())
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
impl DelegateComponent<TransactionStarterComponent> for App {
    type Delegate = BeginPooledTransaction;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<TransactionStarterComponent, __Context__, __Params__> for App
where
    BeginPooledTransaction: IsProviderFor<
        TransactionStarterComponent,
        __Context__,
        __Params__,
    >,
{}
impl DelegateComponent<TransactionCommitterComponent> for App {
    type Delegate = CommitPooledTransaction;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<TransactionCommitterComponent, __Context__, __Params__> for App
where
    CommitPooledTransaction: IsProviderFor<
        TransactionCommitterComponent,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<ErrorTypeProviderComponent, ()> for App {}
impl __CheckApp<DbTypeProviderComponent, ()> for App {}
impl __CheckApp<TransactionTypeProviderComponent, ()> for App {}
impl __CheckApp<TransactionStarterComponent, ()> for App {}
impl __CheckApp<TransactionCommitterComponent, ()> for App {}
fn main() {
    let app = App {
        database: Pool(core::marker::PhantomData),
    };
    app.run_unit_of_work().unwrap();
}
