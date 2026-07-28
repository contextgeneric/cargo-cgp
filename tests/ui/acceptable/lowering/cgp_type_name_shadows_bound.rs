//! Acceptable failure: an abstract type is given the same name as the trait that
//! bounds it, so the bound resolves to the associated type rather than the trait.
//!
//! `#[cgp_type] trait HasDatabaseType { type Database: Database; }` reads as "the
//! abstract type `Database`, which must implement the `Database` trait" — but inside
//! the declaration the name `Database` is already taken by the associated type being
//! declared, so the bound position resolves to that rather than to the trait in
//! scope. The compiler rejects it with `E0404` "expected trait, found type parameter
//! `Database`", the associated type having become a type parameter in the
//! `#[cgp_type]` expansion.
//!
//! The failure is a plain name collision rather than anything CGP-specific, but it
//! is easy to walk into when naming an abstract type after the concrete trait it
//! abstracts over — which is why an abstract database type reads better as `Db`,
//! as in [`ok/abstract_db_transaction.rs`](../../ok/abstract_db_transaction.rs).
//!
//! See cgp-knowledge-base/cgp/reference/macros/cgp_type.md and
//! cgp-knowledge-base/cgp/errors/lowering/out-of-scope-generated-name.md.

use cgp::prelude::*;

pub trait Database: Sized {
    type Row;
}

pub struct Postgres;

impl Database for Postgres {
    type Row = String;
}

#[cgp_type]
pub trait HasDatabaseType {
    type Database: Database;
}

fn main() {}
