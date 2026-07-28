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

#[cgp_type]
pub trait HasDbType {
    type Db: Database;
}

#[cgp_type]
pub trait HasTransactionType {
    type Transaction;
}

#[cgp_component(TransactionStarter)]
#[use_type(HasTransactionType.Transaction, HasErrorType.Error)]
pub trait CanBeginTransaction {
    fn begin_transaction(&self) -> Result<Transaction, Error>;
}

#[cgp_component(TransactionCommitter)]
#[use_type(HasTransactionType.Transaction, HasErrorType.Error)]
pub trait CanCommitTransaction {
    fn commit_transaction(&self, transaction: Transaction) -> Result<(), Error>;
}

#[cgp_impl(new BeginPooledTransaction)]
#[use_type(HasDbType.Db, HasTransactionType.Transaction, HasErrorType.Error)]
impl TransactionStarter
where
    Transaction: CanBeginFrom<Db>,
{
    fn begin_transaction(&self, #[implicit] database: &Pool<Db>) -> Result<Transaction, Error> {
        Ok(<Transaction as CanBeginFrom<Db>>::begin_from(database))
    }
}

#[cgp_impl(new CommitPooledTransaction)]
#[use_type(HasTransactionType.Transaction, HasErrorType.Error)]
impl TransactionCommitter {
    fn commit_transaction(&self, transaction: Transaction) -> Result<(), Error> {
        let _ = transaction;
        Ok(())
    }
}

#[cgp_fn]
#[uses(CanBeginTransaction, CanCommitTransaction)]
#[use_type(HasErrorType.Error)]
pub fn run_unit_of_work(&self) -> Result<(), Error> {
    let transaction = self.begin_transaction()?;
    self.commit_transaction(transaction)?;
    Ok(())
}

#[derive(HasField)]
pub struct App {
    pub database: Pool<Postgres>,
}

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        DbTypeProviderComponent: UseType<Postgres>,
        TransactionTypeProviderComponent: UseType<Tx<Postgres>>,
        TransactionStarterComponent: BeginPooledTransaction,
        TransactionCommitterComponent: CommitPooledTransaction,
    }
}

check_components! {
    App {
        ErrorTypeProviderComponent,
        DbTypeProviderComponent,
        TransactionTypeProviderComponent,
        TransactionStarterComponent,
        TransactionCommitterComponent,
    }
}

fn main() {
    let app = App {
        database: Pool(core::marker::PhantomData),
    };

    app.run_unit_of_work().unwrap();
}
