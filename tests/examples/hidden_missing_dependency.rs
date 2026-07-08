//! A CGP wiring error whose root cause the compiler *hides* — the class `cargo-cgp`
//! most wants to make readable.
//!
//! `GreetHello` needs `Self: HasName`, but `Person` has no `name` field, so the
//! dependency is unmet. The failure is triggered by calling `greet` directly, and
//! there is no `check_components!` to surface it, so the compiler reports only that
//! `greet`'s trait bounds are not satisfied (`E0599`/`E0277`) and never names the
//! missing `name` field. Contrast `greet_ok.rs`, which is correctly wired, and note
//! that adding a `check_components!` block here would instead *surface* the missing
//! `HasName` at the wiring site.
//!
//! See ../../../cgp/docs/errors/hidden/unsatisfied-dependency.md for the anatomy of
//! this error class.

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
    // No `name` field — `GreetHello`'s `Self: HasName` dependency cannot be met.
    pub age: u8,
}

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}

fn main() {
    let person = Person { age: 42 };
    // Hidden-cause error: the compiler says `greet`'s bounds are unsatisfied without
    // naming the missing `name` field.
    person.greet();
}
