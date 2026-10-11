//! A plain Rust error inside a hand-written impl whose CGP supertrait also fails.
//!
//! `Person` is missing the `name` field `GreetHello` reads, so `CanGreetTwice`'s supertrait
//! `CanGreet` fails. The impl body also passes an `Rc` to a `Send` bound, a separate mistake. Plain
//! `cargo check` reports both; the tool re-anchors the `Send` error to the CGP failure and its
//! de-duplication then drops it, so the user never sees an error they must fix.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

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

pub trait CanGreetTwice: CanGreet {
    fn greet_twice(&self) -> String;
}

fn needs_send<T: Send>(_value: T) {}

impl CanGreetTwice for Person {
    fn greet_twice(&self) -> String {
        needs_send(Rc::new(1));
        self.greet()
    }
}

fn main() {}
