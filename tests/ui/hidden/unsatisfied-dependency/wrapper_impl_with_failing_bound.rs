//! A wrapper impl on a context whose supertrait and own `where` bound fail for different causes.
//!
//! `CanGreetTwice` has the consumer trait `CanGreet` as its supertrait, which fails because
//! `Person` lacks the `name` field `GreetHello` reads. The impl also requires `Person: CanWave`, and
//! `Person` wires no waver. rustc reports the supertrait at the impl's `Self` type and the bound at
//! the bound. The tool reports each cause on its own caret: the missing waver wiring on the bound,
//! which the impl-site anchor reads as the consumer obligation it names, and the missing field on
//! the impl's `Self` type, from the supertrait.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E003 CGP-E101 CGP-E102 CGP-E104 CGP-E110 CGP-E113

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_component(Waver)]
pub trait CanWave {
    fn wave(&self) -> String;
}

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_impl(new GreetHello)]
#[uses(HasName)]
impl Greeter {
    fn greet(&self) -> String {
        self.name().to_owned()
    }
}

#[derive(HasField)]
pub struct Person {}

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}

pub trait CanGreetTwice: CanGreet {}

impl CanGreetTwice for Person where Person: CanWave {}

fn main() {}
