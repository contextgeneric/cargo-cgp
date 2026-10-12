//! A wiring key guessed from the consumer trait's name, `CanGreetComponent`.
//!
//! The component of `CanGreet` is named after its provider trait, `GreeterComponent`, so the guessed
//! key is an `E0425` "cannot find type", reported twice. rustc suggests the trait `CanUseComponent`,
//! which is not a component, and the real key is not suggested.
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

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) {}
}

pub struct Person;

delegate_components! {
    Person {
        CanGreetComponent: GreetHello,
    }
}

fn main() {}
