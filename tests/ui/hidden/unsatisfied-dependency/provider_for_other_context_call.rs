//! A provider written for one context, wired on another, and reached by a call.
//!
//! The use-site anchor re-checks `Robot`'s wiring, and the walk reaches `GreetPerson: Greeter<Robot>`,
//! which no impl of `GreetPerson` matches:
//! ``[CGP-E127] `GreetPerson` implements `Greeter` only for `Person`, not `Robot` ``.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E127

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

pub struct Person;
pub struct Robot;

#[cgp_impl(new GreetPerson)]
impl Greeter for Person {
    fn greet(&self) {}
}

delegate_components! {
    Robot {
        GreeterComponent: GreetPerson,
    }
}

fn main() {
    Robot.greet();
}
