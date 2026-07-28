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

#[cgp_fn]
#[use_type(HasDbType.Db, HasTransactionType.Transaction)]
pub fn begin_transaction(&self, #[implicit] database: &Pool<Db>) -> Transaction
where
    Transaction: CanBeginFrom<Db>,
{
    Transaction::begin_from(database)
}

fn main() {}
