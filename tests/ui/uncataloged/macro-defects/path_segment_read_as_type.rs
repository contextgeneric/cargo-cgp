//! An `@`-path segment that `Path!` reads as a type rather than a symbol.
//!
//! `@app.f` lowers `f` to a type, a documented `Path!` defect, so three `E0425`s report it. The
//! overflow that follows keeps rustc's `E0275` wording: the context's wiring holds the unresolved
//! type, so the tool does not report it as a `[CGP-E020]` cycle. Nothing names the `Path!` rule as
//! the cause.
//!
//! CGP error class: none; the `Path!` segment rule is a known issue of the macro.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

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
