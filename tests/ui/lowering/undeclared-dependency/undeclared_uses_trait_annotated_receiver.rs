//! A `#[cgp_impl]` provider body that calls an undeclared trait through an annotated `let`.
//!
//! `let context: &Self = self;` names the context with an annotation, and the call is on that
//! binding. The undeclared-trait detector reads the annotation, so it recognizes this receiver as
//! the context and reports the missing `#[uses]`.

//@quality: acceptable
//@codes: CGP-E030

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
