//! A hidden-class error whose cause the tool surfaces.
//!
//! `GreetHello` needs `Self: HasName`, but `Person` has no `name` field, and the
//! failure is triggered by calling `greet` directly with no `check_components!`.
//! Under a plain `cargo check` this is the hidden class — a bare `E0599` "method not
//! found" that never names the cause. Under the next-generation solver the tool
//! recovers it: one `[CGP-E001]` block over a `[CGP-E110]` root cause naming the missing
//! `name` field, without rustc's advice to use associated-function syntax. A passing
//! snapshot is also the standing proof that the driver stands in as the compiler, since
//! only a compilation through the tool can produce this cause.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md.

//@json
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
