//! Usability: a CGP check failure and an ordinary Rust error in the same crate, to confirm the
//! typed resolver replaces only the CGP wiring diagnostic and leaves the plain error untouched.
//!
//! `Person` is missing the `name` field its `GreetHello` provider needs, which the resolver
//! turns into a dependency tree. The `wrong_type` function has an unrelated `E0308` type
//! mismatch, which is not a check-entry `E0277`, so the resolver declines it and it flows
//! through the fallback pipeline unchanged. Both diagnostics should appear.
//!
//! CGP error class: ../../../../../cgp/docs/errors/checks/check-trait-failure.md.

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
    // missing name field to trigger the CGP check error
}

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}

check_components! {
    Person {
        GreeterComponent,
    }
}

// An ordinary Rust error, unrelated to CGP wiring: the body's type does not match the return
// type (`E0308`). The resolver must leave this diagnostic alone.
fn wrong_type() -> u32 {
    "not a number"
}

fn main() {}
