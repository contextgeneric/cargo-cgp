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

#[cgp_type]
pub trait HasDbType {
    type Db: Database;
}

#[cgp_type]
pub trait HasTransactionType {
    type Transaction;
}

#[cgp_fn]
#[use_type(HasDbType.Db, HasTransactionType.{Transaction = Tx<Db>})]
pub fn begin_transaction(&self, #[implicit] database: &Pool<Db>) -> Transaction {
    let _ = database;
    todo!()
}

#[derive(HasField)]
pub struct App {
    pub database: Pool<Postgres>,
}

delegate_components! {
    App {
        DbTypeProviderComponent: UseType<Postgres>,
        TransactionTypeProviderComponent: UseType<Tx<Postgres>>,
    }
}

check_components! {
    App {
        DbTypeProviderComponent,
        TransactionTypeProviderComponent,
    }
}

// Requiring the capability on `App` is what proves the *pinned* bound is
// satisfiable, not merely well-formed: the blanket impl applies only if `App`'s
// wired `Transaction` really is `Tx<App::Db>`, which is the equality the pin emits.
fn assert_can_begin<Context>()
where
    Context: BeginTransaction,
{
}

fn main() {
    assert_can_begin::<App>();
}
