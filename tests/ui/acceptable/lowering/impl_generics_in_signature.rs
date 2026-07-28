//! Acceptable failure: an `#[impl_generics]` parameter is named in the generated
//! trait's method signature, where it is out of scope.
//!
//! `#[impl_generics(Db: Database)]` adds `Db` to the generated *impl* alone — that
//! is the whole point of the attribute, and it is what lets a context supply the
//! type implicitly through the type of its `database` field, with no parameter on
//! the trait for any caller to thread. But the trait is generated without `Db`, so
//! a method signature that names it — here the return type `Db::Row` — refers to a
//! type the trait cannot see, and the compiler rejects the generated trait with
//! `E0433` "cannot find type `Db` in this scope".
//!
//! This is the forcing condition that separates an inferred type from an abstract
//! one: a type reachable only through an implicit argument may stay an
//! `#[impl_generics]` parameter, while a type the capability *names* in its public
//! signature must be an abstract type the context supplies by wiring. The working
//! counterpart is [`ok/abstract_db_transaction.rs`](../../ok/abstract_db_transaction.rs).
//!
//! See cgp-knowledge-base/cgp/guides/naming-a-type-dependency.md and
//! cgp-knowledge-base/cgp/errors/lowering/out-of-scope-generated-name.md.

use cgp::prelude::*;

pub trait Database: Sized {
    type Row;
}

pub struct Pool<Db>(pub core::marker::PhantomData<Db>);

pub struct Postgres;

impl Database for Postgres {
    type Row = String;
}

#[cgp_fn]
#[impl_generics(Db: Database)]
pub fn fetch_row(&self, #[implicit] database: &Pool<Db>) -> Db::Row {
    let _ = database;
    todo!()
}

fn main() {}
