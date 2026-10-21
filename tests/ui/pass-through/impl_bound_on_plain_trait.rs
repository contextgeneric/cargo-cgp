//! A hand-written impl whose own `where` bound on a context names a plain Rust trait.
//!
//! `Person` is a context, wired with `GreetHello`, and the impl requires `Person: Clone`, which
//! `Person` does not derive. The compiler reports the bound at the bound. Nothing CGP fails, so the
//! tool must print what plain `cargo check` prints: the impl-site anchor reads a failing bound of
//! an impl's own `where` clause only when it names a CGP consumer trait.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.

#![allow(unused_imports)]

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        "hello".to_owned()
    }
}

pub struct Person;

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}

pub trait CanCopyGreeting {}

impl CanCopyGreeting for Person where Person: Clone {}

fn main() {}
