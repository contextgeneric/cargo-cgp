//! A consumer trait written where the component marker belongs as a wiring key.
//!
//! rustc reports `E0782` "expected a type, found a trait" with a suggestion to add `dyn`, which is
//! wrong for a wiring key. The tool reshapes it into `[CGP-E040]`, naming the consumer trait and,
//! in a `help`, its component `GreeterComponent`, read off the trait's provider trait.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: CGP-E040

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
        CanGreet: GreetHello,
    }
}

fn main() {}
