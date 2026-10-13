//! An `@`-path segment naming a type that was never declared, redirected to and wired.
//!
//! `@app.Missing` keeps `Missing` as a type segment, which does not resolve, so rustc reports an
//! `E0425` at each use. The check that follows overflows on the lookup through the unresolved key,
//! and the tool keeps rustc's `E0275` wording rather than reporting a `[CGP-E020]` cycle: the wiring
//! holds a `{type error}`, the earlier error is the one to fix, and the wiring has no cycle.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

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
        GreeterComponent => @app.Missing,
        @app.Missing: GreetHello,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
