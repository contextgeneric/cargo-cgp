//! An `@`-path segment that misspells a declared type.
//!
//! The compiler's `E0425` carries the right suggestion, the similarly named struct, beside the
//! "missing a type parameter" suggestion that no wiring entry can take. The tool reshapes the header
//! into `[CGP-E044]`, drops the type-parameter suggestion, keeps the near miss, and adds no help of
//! its own, since the near miss already says what to write.
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

pub struct Square;

delegate_components! {
    App {
        GreeterComponent => @app.Squre,
        @app.Square: GreetHello,
    }
}

fn main() {}
