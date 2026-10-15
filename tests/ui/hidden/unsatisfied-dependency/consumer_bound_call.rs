//! A consumer trait required as a generic function's bound, called with a context whose provider
//! needs a field it lacks.
//!
//! `greet_twice(&person)` requires `Person: CanGreet`, whose provider `GreetHello` reads a `name`
//! field `Person` does not have. The bound-call anchor reads the bound off `greet_twice`'s own
//! `where` clauses, with `Person` from the argument, and walks it from the consumer:
//! ``[CGP-E001] missing field `name` on `Person` ``, its tree headed by `CanGreet`.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E104 CGP-E110

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
#[uses(HasName)]
impl Greeter {
    fn greet(&self) {
        let _ = self.name();
    }
}

#[derive(HasField)]
pub struct Person {
    pub age: u8,
}

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}

fn greet_twice<Context: CanGreet>(context: &Context) {
    context.greet();
    context.greet();
}

fn main() {
    let person = Person { age: 0 };
    greet_twice(&person);
}
