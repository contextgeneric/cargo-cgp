//! A plain Rust error inside an argument of a consumer method call whose wiring fails.
//!
//! `Person` is missing the `name` field `GreetHello` reads, so `person.greet(...)` fails. Its
//! argument passes an `Rc` to a `Send` bound, a separate mistake inside the call. The tool reports
//! both, as plain `cargo check` does: the `[CGP-E001]` block with its caret on `greet`, and the
//! `Send` error as rustc wrote it. The call-site anchor takes a call only when the diagnostic's
//! primary span is on the call itself, its method name, or a whole argument; the `Send` error's
//! span lies strictly inside the argument `count(Rc::new(1))`, so it stays with that code.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E110

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
