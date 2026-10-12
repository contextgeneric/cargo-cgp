//! Two `#[prefix]` attributes registering one component into one namespace.
//!
//! `[CGP-E014]` names the namespace and both redirect paths, with both carets on `#[cgp_component]`.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).
//!
//! `cgp` gives both generated registrations the attribute's span, so the carets cannot split.

//@quality: acceptable
//@codes: CGP-E014

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
