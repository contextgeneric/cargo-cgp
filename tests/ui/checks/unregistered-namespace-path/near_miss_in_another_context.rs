//! Two contexts miss `@app.GreeterComponent`, and only `Other` wires the near miss
//! `@App.GreeterComponent`.
//!
//! Each block is `[CGP-E113]` on `@app.GreeterComponent`. `Other`'s carries a `help` naming its
//! near-miss entry; `App`'s carries none, since a near miss is looked for in the missing key's own
//! context, and `App` wires no similar key.
//!
//! CGP error class: unregistered namespace path (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/unregistered-namespace-path.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E103 CGP-E113

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
    }
}

pub struct Other;

delegate_components! {
    Other {
        namespace DefaultNamespace;

        @App.GreeterComponent: GreetHello,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

check_components! {
    Other {
        GreeterComponent,
    }
}

fn main() {}
