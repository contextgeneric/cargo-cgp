//! A minimal, correctly-wired CGP program. `cargo cgp check --example greet_ok`
//! (or `scripts/run-check.sh greet_ok`) should succeed with no errors — this is the
//! baseline that the error examples contrast against.
//!
//! `Person` carries a `name` field, so `GreetHello`'s `Self: HasName` dependency is
//! satisfied and `check_components!` passes.

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

fn main() {
    let person = Person {
        name: "World".to_owned(),
    };
    person.greet();
}
