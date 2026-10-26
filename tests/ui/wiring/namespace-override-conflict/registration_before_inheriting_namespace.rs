//! A provider's `#[default_impl]` registers a bare component key into a namespace written after
//! it, whose parent already routes that component under its `#[prefix]` path. The registration and
//! the namespace's inheritance blanket conflict with the coherence error E0119, and rustc names the
//! registration as the first implementation and the blanket as the conflicting one.
//!
//! The tool reads the blanket's redirect for the registered key as it does when the namespace comes
//! first ([namespace_inherited_unprefixed_key.rs](namespace_inherited_unprefixed_key.rs)):
//! `[CGP-E013] conflicting wiring: `AppNamespace` wires component `GreeterComponent` directly, but
//! it is redirected to `@app.GreeterComponent``, with the help naming the path key.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/namespace-override-conflict.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E013).

//@quality: acceptable
//@codes: CGP-E013

use cgp::prelude::*;

#[cgp_component(Greeter)]
#[prefix(@app in DefaultNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
#[default_impl(GreeterComponent in AppNamespace)]
impl Greeter {
    fn greet(&self) -> String {
        "Hello".to_owned()
    }
}

cgp_namespace! {
    new AppNamespace: DefaultNamespace
}

fn main() {}
