//! A wiring key guessed from the consumer trait's name, `CanGreetComponent`.
//!
//! The component of `CanGreet` is named after its provider trait, `GreeterComponent`, so the guessed
//! key is an `E0425` "cannot find type", reported twice, with a suggestion of the trait
//! `CanUseComponent`. The tool reshapes it into one `[CGP-E042]` whose `help` names the real
//! component of `CanGreet`.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: CGP-E042

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
