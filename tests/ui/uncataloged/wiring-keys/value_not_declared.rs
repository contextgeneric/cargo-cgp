//! A wiring value naming a provider that was never declared.
//!
//! rustc's `E0425` suggests writing the provider trait `Greeter` as the value, which is wrong. The
//! tool reshapes it into `[CGP-E043]`, naming the provider and the component it is wired for, and
//! drops the suggestion. No provider of `GreeterComponent` has a similar name, so it adds no `help`.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: CGP-E043

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
        GreeterComponent: GreetHi,
    }
}

fn main() {}
