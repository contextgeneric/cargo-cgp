//! `#[uses]` on a `#[cgp_component]` trait, where `#[extend]` belongs.
//!
//! The unresolved attribute keeps rustc's suggestion of `#[used]`.
//!
//! CGP error class: wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

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
