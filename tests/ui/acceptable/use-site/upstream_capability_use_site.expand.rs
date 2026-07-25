#![feature(prelude_import)]
//! A `#[cgp_fn]` capability defined **upstream**, called on a context that cannot satisfy it.
//!
//! This is `acceptable/use-site/cgp_fn_use_site.rs` with one thing changed: the capability
//! lives in another crate. That is the normal arrangement — a library publishes capabilities
//! and an application consumes them — and it is the arrangement the recognition used to miss.
//!
//! A `#[cgp_fn]`/`#[blanket_trait]` capability is not a CGP component, so no marker or
//! provider trait identifies it; the resolver recognizes one by its blanket impl over a bare
//! context. A blanket impl alone is far too broad a signal — `ToString`, `Into`, and `Borrow`
//! all have one — so recognition was gated to traits the checked crate defines, which excluded
//! every published capability along with the std blankets it was aimed at, and the
//! `[CGP-E009]` reshaping stopped the moment a capability moved into a library.
//!
//! A foreign trait now qualifies on positive evidence instead: its blanket must depend on a CGP
//! construct, followed through composed capabilities. Here `Describe` depends on `HasName`,
//! whose own blanket depends on `HasField` — so the chain is CGP's, and the call resolves to
//! the same `[CGP-E009]` block over the missing field its local twin gives, rather than falling
//! through to rustc's `E0599` with the cause in a mid-stack note under the method-probe
//! candidate list. `ToString` reaches no such construct and stays excluded.
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
