//! A plain Rust error inside a hand-written impl whose CGP supertrait also fails.
//!
//! `Person` is missing the `name` field `GreetHello` reads, so `CanGreetTwice`'s supertrait
//! `CanGreet` fails. The impl body also passes an `Rc` to a `Send` bound, a separate mistake. The
//! tool reports both, as plain `cargo check` does: the `[CGP-E003]` block on the impl, and the
//! `Send` error as rustc wrote it. The impl-site anchor takes a caret inside a method body only on a
//! call to a method of the impl's trait or its supertraits, so the forwarding `self.greet()` error
//! joins the CGP block while the `Send` error keeps its own.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E003 CGP-E101 CGP-E102 CGP-E104 CGP-E110

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
