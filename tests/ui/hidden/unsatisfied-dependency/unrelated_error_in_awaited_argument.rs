//! A plain Rust error inside an argument of an awaited consumer method call whose wiring fails.
//!
//! `Person` is missing the `name` field `GreetHello` reads, so the async `person.greet(...)` fails.
//! Its argument passes an `Rc` to a `Send` bound, a separate mistake inside the awaited call. The
//! tool reports both, as plain `cargo check` does. Awaiting the call changes nothing: the call-site
//! anchor takes the call only for a primary span on the call, its method name, or a whole argument,
//! and the `Send` error's span lies strictly inside the argument `count(Rc::new(1))`.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E110

use std::rc::Rc;

use cgp::prelude::*;

#[async_trait]
#[cgp_component(Greeter)]
pub trait CanGreet {
    async fn greet(&self, times: u32) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    async fn greet(&self, times: u32, #[implicit] name: &str) -> String {
        format!("Hello, {name} x{times}")
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

fn count<T: Send>(_value: T) -> u32 {
    1
}

pub async fn run(person: &Person) -> String {
    person.greet(count(Rc::new(1))).await
}

fn main() {}
