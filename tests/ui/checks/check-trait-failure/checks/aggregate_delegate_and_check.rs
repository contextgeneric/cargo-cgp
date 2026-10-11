//! `delegate_and_check_components!` on a `new` provider bundle.
//!
//! The check blames the bundle (`[CGP-E106] missing field `name` on `GreetBundle``); nothing says a
//! provider bundle was checked as a context.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E106

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self, #[implicit] name: &str) -> String {
        format!("hello {name}")
    }
}

delegate_and_check_components! {
    new GreetBundle {
        GreeterComponent: GreetHello,
    }
}

fn main() {}
