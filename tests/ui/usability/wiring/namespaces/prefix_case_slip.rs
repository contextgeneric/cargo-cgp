//! `@App.GreeterComponent` wired where the component's prefix is `@app`.
//!
//! `[CGP-E107]` on `@app.GreeterComponent`; the near-miss entry is not pointed out.
//!
//! CGP error class: unregistered namespace path (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/unregistered-namespace-path.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

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

        @App.GreeterComponent: GreetHello,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
