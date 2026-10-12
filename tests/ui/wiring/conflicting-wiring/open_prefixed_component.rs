//! `open` on a component that is prefixed into a joined namespace.
//!
//! The namespace already redirects `GreeterComponent` to `@app.GreeterComponent`, so `open`
//! redirects it a second time. `[CGP-E014]` names both targets; the fix is to drop `open` and wire
//! `@app.GreeterComponent: GreetHello`.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).

//@quality: acceptable
//@codes: CGP-E014

use cgp::prelude::*;

#[cgp_component(Greeter)]
#[prefix(@app in DefaultNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        "hello".to_owned()
    }
}

pub struct App;

delegate_components! {
    App {
        namespace DefaultNamespace;
        open GreeterComponent;

        @GreeterComponent: GreetHello,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
