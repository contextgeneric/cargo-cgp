//! A wiring value that names no provider, beside a component whose wiring loops and whose consumer
//! trait's name starts with the first component's.
//!
//! `GreeterComponent: GreetHi` names no type, so rustc reports an `E0425` at the value, and the
//! entry leaves `Person: CanGreet` unprovable. `TwiceGreeterComponent: UseContext` leads back to
//! itself, so the call `Person.greet_twice()` overflows on `Person: CanGreetTwice`, a requirement
//! that begins with `Person: CanGreet` but is not the one the first entry leaves unprovable. The tool
//! reports `[CGP-E043]` at the value and keeps the overflow, reshaped as the cycle it is.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: CGP-E020 CGP-E043

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_component(TwiceGreeter)]
pub trait CanGreetTwice {
    fn greet_twice(&self);
}

pub struct Person;

delegate_components! {
    Person {
        GreeterComponent: GreetHi,
        TwiceGreeterComponent: UseContext,
    }
}

fn main() {
    Person.greet_twice();
}
