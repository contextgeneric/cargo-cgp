//! A plain Rust error inside an argument of a consumer method call whose wiring fails.
//!
//! `Person` is missing the `name` field `GreetHello` reads, so `person.greet(...)` fails. Its
//! argument passes an `Rc` to a `Send` bound, a separate mistake inside the call. Plain
//! `cargo check` reports both; the tool reports only the `[CGP-E001]` block, and the `Send` error
//! is lost.
//!
//! This is a known limitation of the call-site anchor. It matches a call by the diagnostic's
//! primary span, and the `Send` error's span lies inside the call's argument, so the anchor reads it
//! as another report of the failing call, resolves it to the same cause, and de-duplicates it away.
//! A CGP failure can also point into an argument (the `PhantomData` tag of a handler call), and
//! rustc's `E0277` carries no structured predicate to tell the two apart.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: hidden
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E106

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

fn count<T: Send>(_value: T) -> u32 {
    1
}

fn run(person: &Person) -> String {
    person.greet(count(Rc::new(1)))
}

fn main() {
    let _ = run(&Person { age: 3 });
}
