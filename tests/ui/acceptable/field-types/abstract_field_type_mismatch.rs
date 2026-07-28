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

#[cgp_type]
pub trait HasDbType {
    type Db: Database;
}

#[cgp_component(RowCounter)]
#[use_type(HasErrorType.Error)]
pub trait CanCountRows {
    fn count_rows(&self) -> Result<u64, Error>;
}

#[cgp_impl(new CountPooledRows)]
#[use_type(HasDbType.Db, HasErrorType.Error)]
impl RowCounter {
    fn count_rows(&self, #[implicit] database: &Pool<Db>) -> Result<u64, Error> {
        let _ = database;
        Ok(0)
    }
}

#[derive(HasField)]
pub struct App {
    pub database: Pool<Sqlite>,
}

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        DbTypeProviderComponent: UseType<Postgres>,
        RowCounterComponent: CountPooledRows,
    }
}

check_components! {
    App {
        RowCounterComponent,
    }
}

fn main() {}
