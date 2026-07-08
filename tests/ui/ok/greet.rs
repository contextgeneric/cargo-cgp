//! A minimal, correctly-wired CGP program — the passing baseline the error cases
//! contrast against. `Person` carries a `name` field, so `GreetHello`'s
//! `Self: HasName` dependency is satisfied and `check_components!` passes, so the
//! snapshot of cargo-cgp's output is empty.

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
