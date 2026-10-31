//! A `#[cgp_impl]` provider body that calls an undeclared trait through an annotated `let`.
//!
//! `let context: &Self = self;` names the context with an annotation, and the call is on that
//! binding. The undeclared-trait detector follows only an untyped forwarding `let` back to the
//! parameter it copies, so it does not recognize this receiver.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_fn]
fn person_name(&self, #[implicit] name: &str) -> String {
    name.to_owned()
}

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        let context: &Self = self;
        format!("Hello, {}!", context.person_name())
    }
}

fn main() {}
