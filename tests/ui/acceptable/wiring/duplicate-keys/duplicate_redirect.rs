//! Acceptable failure: the same component opened for redirection twice. Each
//! `open GreeterComponent;` lowers to a `DelegateComponent<GreeterComponent>` whose
//! `Delegate` is a `RedirectLookup`, so the two conflict with the coherence error
//! E0119 — a *duplicate redirect*, distinct from a redirect that collides with a
//! direct wiring ([duplicate_open_key.rs]).
//!
//! The tool recognizes that *both* conflicting entries redirect the same key: it
//! drops the redundant `IsProviderFor` half and rewrites the `DelegateComponent`
//! half to `[CGP-E004] duplicate redirect for component `GreeterComponent` …`,
//! naming the redirected path, while keeping rustc's two carets on the `open`
//! lines. If the redirect detection in `resolve/conflict.rs` regresses (counting
//! one redirect instead of two), the header reverts to the single-redirect form.
//!
//! See docs/errors/wiring/conflicting-wiring.md and docs/error-code.md (CGP-E004).

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) {}
}

pub struct Person;

delegate_components! {
    Person {
        open GreeterComponent;
        open GreeterComponent;
    }
}

fn main() {}
