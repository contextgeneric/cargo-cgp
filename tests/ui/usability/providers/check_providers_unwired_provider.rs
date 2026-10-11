//! `#[check_providers(GreetHello)]` on a fieldless context.
//!
//! A raw `E0277` on `App: HasField<Symbol!("name")>` with the derive help, no code and no tree.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

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
