//! A `check_components!` failure whose root cause is a provider written for another context:
//! `GreetPerson` implements `Greeter` for `Person` alone, and `Robot` wires it.
//!
//! The walk reaches `GreetPerson: Greeter<Robot>`, which no impl of `GreetPerson` matches, and
//! names the context the provider takes:
//! ``[CGP-E127] `GreetPerson` implements `Greeter` only for `Person`, not `Robot` ``.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

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

check_components! {
    Robot {
        GreeterComponent,
    }
}

fn main() {}
