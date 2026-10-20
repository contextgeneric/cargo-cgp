//! `delegate_and_check_components!` on a `new` provider bundle.
//!
//! The macro checks the bundle as the context of its own entries, so the check reports
//! ``[CGP-E001] missing field `name` on `GreetBundle` ``. That is true of the check as written: a
//! `new` table also declares a fieldless context, and neither the macro nor the tool can tell the
//! two apart, so nothing says that a bundle is checked through a context that delegates to it.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E110

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
