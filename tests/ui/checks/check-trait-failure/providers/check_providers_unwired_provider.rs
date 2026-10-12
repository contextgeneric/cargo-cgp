//! `#[check_providers(GreetHello)]` on a fieldless context.
//!
//! The entry checks `GreetHello` directly, whatever `App` wires, so the tool anchors on it and walks
//! `GreetHello: Greeter<App>`: a `[CGP-E002]` block stating the missing `name` field over a tree that
//! starts at the provider. Plain rustc reports the `HasField` bound with a `#[derive(HasField)]` help
//! that does not fit a struct without the field.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E002 CGP-E102 CGP-E110

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

pub struct App;

check_components! {
    #[check_providers(GreetHello)]
    App {
        GreeterComponent,
    }
}

fn main() {}
