//! Calling the optional builder's `set` on a core builder.
//!
//! `Context::builder().set(…)` is an `E0271` on `Mapper == IsOptional`. The output never suggests
//! starting from `optional_builder()`, which is what `set` needs.
//!
//! CGP error class: none yet; the extensible-data family
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/concepts/extensible-records.md)
//! is not in the upstream catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use cgp::extra::field::impls::SetOptional;
use cgp::prelude::*;

#[derive(CgpData)]
pub struct Context {
    pub foo: String,
    pub bar: u64,
}

fn main() {
    let _ = Context::builder().set(PhantomData::<Symbol!("foo")>, "foo".to_owned());
}
