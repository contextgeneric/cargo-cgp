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

#[cgp_type]
pub trait HasDbType {
    type Db: Database;
}

#[cgp_type]
pub trait HasTransactionType {
    type Transaction;
}

#[cgp_component(TransactionNamer)]
#[use_type(HasTransactionType.Transaction)]
pub trait CanNameTransaction {
    fn name_transaction(&self, transaction: Transaction) -> &'static str;
}

// The provider pins `Transaction` to a transaction *of the context's own database type*,
// so its requirement is a projection rather than a constant.
#[cgp_impl(new NamePooledTransaction)]
#[use_type(HasDbType.Db, HasTransactionType.{Transaction = Tx<Db>})]
impl TransactionNamer {
    fn name_transaction(&self, transaction: Transaction) -> &'static str {
        let _ = transaction;
        "pooled"
    }
}

pub struct App;

delegate_components! {
    App {
        DbTypeProviderComponent: UseType<Postgres>,
        // Disagrees with the pin: the provider requires `Tx<<App as HasDbType>::Db>`,
        // which reduces to `Tx<Postgres>`.
        TransactionTypeProviderComponent: UseType<Tx<Sqlite>>,
        TransactionNamerComponent: NamePooledTransaction,
    }
}

check_components! {
    App {
        TransactionNamerComponent,
    }
}

fn main() {}
