//! A wiring value that misspells a provider of the key's component, beside a closer name that
//! provides another component.
//!
//! `GreeterComponent: GreetHelo` names no type, so rustc reports an `E0425` and suggests a
//! similarly named item. The tool reports `[CGP-E043]`, and its help names the provider of
//! `Greeter` with a similar name, `GreetHello`. `Greethelo` matches the misspelling more closely,
//! ignoring case, but it provides `Farewell`, so it is not offered.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: CGP-E043

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_component(Farewell)]
pub trait CanSayBye {
    fn say_bye(&self);
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) {}
}

#[cgp_impl(new Greethelo)]
impl Farewell {
    fn say_bye(&self) {}
}

pub struct Person;

delegate_components! {
    Person {
        GreeterComponent: GreetHelo,
        FarewellComponent: Greethelo,
    }
}

fn main() {}
