//! A wrapper impl on a context whose supertrait and own `where` bound fail for different causes.
//!
//! `CanGreetTwice` has the consumer trait `CanGreet` as its supertrait, which fails because
//! `Person` lacks the `name` field `GreetHello` reads. The impl also requires `Person: CanWave`, and
//! `Person` wires no waver. rustc reports the supertrait at the impl's `Self` type and the bound at
//! the bound. The tool reports one block, at the bound, naming the supertrait's missing field, and
//! drops the missing waver wiring: the impl-site anchor reads every caret on the impl's header as
//! the supertrait's failure.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: usability
//@codes: CGP-E003 CGP-E101 CGP-E102 CGP-E104 CGP-E110

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
