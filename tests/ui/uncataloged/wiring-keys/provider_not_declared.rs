//! A provider used in wiring but never declared, then called.
//!
//! `#[cgp_impl(GreetHello)]` without `new` implements the provider trait for a struct nothing
//! declares, so rustc reports `E0425` at the attribute and again at the wiring value, and the call
//! then overflows (`E0275` "overflow evaluating the requirement `Person: CanGreet`"). The tool
//! reshapes the first into `[CGP-E038]` with `#[cgp_impl(new GreetHello)]` in a `help`, and drops
//! the second and the overflow as its consequences.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: CGP-E038

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_impl(GreetHello)]
impl Greeter {
    fn greet(&self) {}
}

pub struct Person;

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}

fn main() {
    Person.greet();
}
