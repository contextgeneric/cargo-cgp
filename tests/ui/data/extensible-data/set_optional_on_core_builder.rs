//! Calling the optional builder's `set` on a core builder.
//!
//! `Context::builder().set(…)` is an `E0271` on `Mapper == IsOptional`. The output never suggests
//! starting from `optional_builder()`, which is what `set` needs.
//!
//! CGP error class: extensible data (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/data/extensible-data.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

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
