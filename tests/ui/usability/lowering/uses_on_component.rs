//! `#[uses]` on a `#[cgp_component]` trait, where `#[extend]` belongs.
//!
//! The unresolved attribute keeps rustc's suggestion of `#[used]`.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_component(Greeter)]
#[uses(HasName)]
pub trait CanGreet {
    fn greet(&self);
}

fn main() {}
