//! A provider used in wiring but never declared, then called.
//!
//! Two `E0425`s and a cascade `E0275` "overflow evaluating `Person: CanGreet`" at the call, all kept;
//! the overflow is a consequence of the unresolved name.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

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
