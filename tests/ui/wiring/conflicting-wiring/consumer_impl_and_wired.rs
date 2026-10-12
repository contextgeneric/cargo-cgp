//! A consumer trait implemented by hand on a context that also wires the component.
//!
//! The `E0119` puts its caret on `#[cgp_component]` and "first implementation" on the hand impl; the
//! wiring entry that causes the overlap is never named.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: acceptable
//@codes: none

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_impl(new GreetHello)]
impl Greeter
where
    Self: HasName,
{
    fn greet(&self) {
        println!("Hello, {}!", self.name());
    }
}

#[derive(HasField)]
pub struct Person {
    pub name: String,
}

impl CanGreet for Person {
    fn greet(&self) {}
}

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}

fn main() {}
