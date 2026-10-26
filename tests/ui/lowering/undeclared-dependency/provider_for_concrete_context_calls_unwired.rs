//! A provider written for one context, `impl Greeter for Person`, whose body calls a consumer trait
//! `Person` does not wire.
//!
//! The provider's context is the concrete `Person`, not a generated `__Context__`, so the call is not
//! an undeclared dependency: `#[uses(CanWave)]` would only restate a bound `Person` fails. The tool
//! resolves the call through the call-site anchor instead, to the missing wiring of
//! `WaverComponent` on `Person`.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E113

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_component(Waver)]
pub trait CanWave {
    fn wave(&self) -> String;
}

#[derive(HasField)]
pub struct Person {
    pub name: String,
}

#[cgp_impl(new GreetPerson)]
impl Greeter for Person {
    fn greet(&self) -> String {
        self.wave()
    }
}

delegate_components! {
    Person {
        GreeterComponent: GreetPerson,
    }
}

fn main() {}
