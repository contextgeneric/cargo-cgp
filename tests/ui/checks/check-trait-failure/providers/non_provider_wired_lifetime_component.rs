//! A plain type wired where a provider belongs, for a component with a lifetime parameter.
//!
//! `NotAGreeter` has no `Greeter` impl, so a call through the wiring fails on its missing provider
//! trait, as it does for a component without parameters. The provider trait takes the component's
//! lifetime ahead of the context, so a reader that took the context by raw position would land on
//! the lifetime; the tool reads it by type position and names `NotAGreeter` as no provider of
//! `Greeter`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E115

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet<'a> {
    fn greet(&self, other: &'a str) -> String;
}

pub struct NotAGreeter;

pub struct App;

// MISTAKE: `NotAGreeter` is wired where a `Greeter` provider belongs.
delegate_components! {
    App {
        GreeterComponent: NotAGreeter,
    }
}

fn main() {
    let app = App;
    let _ = app.greet("Ada");
}
