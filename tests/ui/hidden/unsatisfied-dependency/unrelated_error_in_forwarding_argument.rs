//! A plain Rust error inside an argument of a hand-written impl's forwarding call, whose CGP
//! supertrait also fails.
//!
//! `Person` is missing the `name` field `GreetHello` reads, so `CanGreetTwice`'s supertrait
//! `CanGreet` fails. The forwarding `self.greet(...)` passes an `Rc` to a `Send` bound in its
//! argument, a separate mistake inside the call. The tool reports both, as plain `cargo check` does:
//! the `[CGP-E003]` block on the impl, and the `Send` error as rustc wrote it. The impl-site anchor
//! takes a caret inside a method body only on a call to a method of the impl's trait or its
//! supertraits, by the rule `unrelated_error_in_call_argument` describes, so a caret strictly
//! inside the forwarding call's argument stays with that code.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E003 CGP-E101 CGP-E102 CGP-E104 CGP-E110

use std::rc::Rc;

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self, times: u32) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self, times: u32, #[implicit] name: &str) -> String {
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

pub trait CanGreetTwice: CanGreet {
    fn greet_twice(&self) -> String;
}

fn count<T: Send>(_value: T) -> u32 {
    1
}

impl CanGreetTwice for Person {
    fn greet_twice(&self) -> String {
        self.greet(count(Rc::new(1)))
    }
}

fn main() {}
