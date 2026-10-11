//! Two `#[prefix]` attributes registering one component into one namespace.
//!
//! `E0119` with both carets on `#[cgp_component]`, passed through uncoded.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

cgp_namespace! {
    new AppNamespace
}

#[cgp_component(Greeter)]
#[prefix(@app in AppNamespace)]
#[prefix(@other in AppNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

fn main() {}
