//! A component wired to `UseContext` with no other implementation, reached by a method call.
//!
//! The call fails with an `E0275` "overflow evaluating the requirement `Person: CanGreet`", on the
//! consumer trait itself rather than the `CanUseComponent` bound a check reports. The tool walks the
//! call's consumer obligation as it walks a check entry's, so the block is `[CGP-E020]` with the
//! help naming `UseContext`, as the check in `use_context_cycle` is.
//!
//! CGP error class: wiring cycle (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/wiring-cycle.md).

//@quality: acceptable
//@codes: CGP-E020

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
