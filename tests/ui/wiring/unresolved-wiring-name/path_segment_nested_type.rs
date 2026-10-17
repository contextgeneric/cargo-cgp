//! An undeclared type nested in the generic arguments of an `@`-path segment.
//!
//! `Missing` is not a segment of the path by itself, so a lowercase spelling would not make it a
//! symbol. The tool reshapes the `E0425` into `[CGP-E044]` and offers no help, since the fix depends
//! on what the segment `Vec<…>` is meant to hold.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: CGP-E044

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
        GreeterComponent: GreetHello,
        @app.Vec<Missing>: GreetHello,
    }
}

fn main() {}
