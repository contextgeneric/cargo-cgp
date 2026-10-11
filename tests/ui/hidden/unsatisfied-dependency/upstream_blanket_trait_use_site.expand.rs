#![feature(prelude_import)]
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
//! same `[CGP-E009]` block over the missing field its local twin gives, rather than to rustc's
//! `E0599` with the cause in a mid-stack note. `ToString` reaches no such construct and stays
//! excluded.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
use cgp_test_crate_a::Describe;
pub struct App {
    pub count: u64,
}
impl HasField<Symbol!("count")> for App {
    type Value = u64;
    fn get_field(
        &self,
        key: ::core::marker::PhantomData<Symbol!("count")>,
    ) -> &Self::Value {
        &self.count
    }
}
impl HasFieldMut<Symbol!("count")> for App {
    fn get_field_mut(
        &mut self,
        key: ::core::marker::PhantomData<Symbol!("count")>,
    ) -> &mut Self::Value {
        &mut self.count
    }
}
fn describe_app(app: &App) -> String {
    app.describe()
}
fn main() {}
