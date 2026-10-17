//! An `@`-path segment naming a type that was never declared, redirected to and wired.
//!
//! `@app.Missing` keeps `Missing` as a type segment, which does not resolve, so the compiler reports
//! an `E0425` at each use, the first with a "missing a type parameter" suggestion no wiring entry
//! can take. The tool reshapes the first into `[CGP-E044]`, which names the type and says a
//! lowercase segment would be a symbol, and drops the repeat and the suggestion. The `E0275` the
//! check then reports on the lookup through the unresolved key is the entry's consequence, so it
//! drops as well.
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
