//! A component wired to `UseContext` with no other implementation, reached by a method call.
//!
//! The call fails with a raw `E0275` "overflow evaluating `Person: CanGreet`", passed through. The
//! `[CGP-E020]` rewrite fires only on a `CanUseComponent` overflow at a check.
//!
//! CGP error class: wiring cycle (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/wiring-cycle.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
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

pub struct Person;

delegate_components! {
    Person {
        GreeterComponent: UseContext,
    }
}

fn main() {
    Person.greet();
}
