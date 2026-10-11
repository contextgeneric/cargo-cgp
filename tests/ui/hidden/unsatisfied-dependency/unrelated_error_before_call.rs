//! A plain Rust error in a function that also calls a consumer method whose wiring fails.
//!
//! `Person` is missing the `name` field `GreetHello` reads, so `person.greet()` fails. The line
//! before it passes an `Rc` to a `Send` bound, a separate mistake. Plain `cargo check` reports both;
//! the tool re-anchors the `Send` error to the failing call, prints one `[CGP-E001]` block with its
//! caret on the `Rc` line, and drops the `Send` error.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E106

use std::rc::Rc;

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) -> String {
        format!("Hello, {name}")
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

fn needs_send<T: Send>(_value: T) {}

fn run(person: &Person) -> String {
    needs_send(Rc::new(1));
    person.greet()
}

fn main() {
    let _ = run(&Person { age: 3 });
}
