//! An `@`-path segment that `Path!` reads as a type rather than a symbol.
//!
//! `@app.f` lowers `f` to a type, a documented `Path!` defect, so three `E0425`s report it. The
//! overflow that follows is rewritten into `[CGP-E010]` with the `UseContext` cycle advice, which is
//! false: the lookup fails because of the unresolved type, not a cycle.
//!
//! CGP error class: none; the `Path!` segment rule is a known issue of the macro.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        "hello".into()
    }
}

pub struct App;

delegate_components! {
    App {
        GreeterComponent => @app.f,
        @app.f: GreetHello,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
