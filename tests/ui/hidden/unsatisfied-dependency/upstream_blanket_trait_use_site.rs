//! A `#[cgp_fn]` trait defined **upstream**, called on a context that cannot satisfy it.
//!
//! This is `cgp_fn_use_site` with one thing changed: the trait lives in another crate. That is
//! the normal arrangement, since a library publishes traits and an application consumes them.
//!
//! A `#[cgp_fn]`/`#[blanket_trait]` blanket trait is not a CGP component, so no marker or
//! provider trait identifies it; the resolver recognizes one by its blanket impl over a bare
//! context. A blanket impl alone is far too broad a signal (`ToString`, `Into`, and `Borrow`
//! all have one), so a foreign trait qualifies only on positive evidence: its blanket must depend
//! on a CGP construct, followed through composed traits. Here `Describe` depends on `HasName`,
//! whose own blanket depends on `HasField`, so the chain is CGP's, and the call resolves to the
//! same `[CGP-E003]` block over the missing field its local twin gives, rather than to rustc's
//! `E0599` with the cause in a mid-stack note. `ToString` reaches no such construct and stays
//! excluded.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md

//@aux-build: cgp-test-crate-a
//@quality: acceptable
//@codes: CGP-E003 CGP-E104 CGP-E110

use cgp::prelude::*;
use cgp_test_crate_a::Describe;

#[derive(HasField)]
pub struct App {
    // No `name` field, so the upstream `HasName` the trait composes cannot hold.
    pub count: u64,
}

fn describe_app(app: &App) -> String {
    app.describe()
}

fn main() {}
