//! A provider needing two prefixed components a context leaves unwired, the second with a near
//! miss among the paths the context does wire.
//!
//! `GreetWithName` requires `CanGreet` and `CanName`, both registered under `@app`. `App` wires
//! neither `@app.GreeterComponent` nor `@app.NamerComponent`, but wires `@App.NamerComponent`, a
//! slip of the second. Both causes are missing wiring on `App`, so their fixes merge into one
//! `help` at the first cause's place, and the near miss of the second cause qualifies that merged
//! fix, so it comes before it, as a near miss comes before its cause's fix.
//!
//! CGP error class: unregistered namespace path (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/unregistered-namespace-path.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E103 CGP-E113

use cgp::prelude::*;

#[cgp_component(Greeter)]
#[prefix(@app in DefaultNamespace)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_component(Namer)]
#[prefix(@app in DefaultNamespace)]
pub trait CanName {
    fn name(&self) -> String;
}

#[cgp_component(Welcomer)]
pub trait CanWelcome {
    fn welcome(&self) -> String;
}

#[cgp_impl(new GreetWithName)]
#[uses(CanGreet, CanName)]
impl Welcomer {
    fn welcome(&self) -> String {
        format!("{} {}", self.greet(), self.name())
    }
}

#[cgp_impl(new NameFixed)]
impl Namer {
    fn name(&self) -> String {
        "app".to_owned()
    }
}

pub struct App;

// MISTAKE: `@app.GreeterComponent` is not wired, and `@App.NamerComponent` slips the prefix.
delegate_components! {
    App {
        namespace DefaultNamespace;

        WelcomerComponent: GreetWithName,
        @App.NamerComponent: NameFixed,
    }
}

check_components! {
    App {
        WelcomerComponent,
    }
}

fn main() {}
