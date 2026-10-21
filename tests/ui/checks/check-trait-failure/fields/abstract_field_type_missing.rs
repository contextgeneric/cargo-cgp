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

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E110

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
    pub name: String,
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
