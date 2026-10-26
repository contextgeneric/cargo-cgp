//! `UseDelegate` wired to a component that takes no parameters.
//!
//! `UseDelegate<GreeterTable>` dispatches on a component's parameters, and `Greeter` has none and
//! no `#[derive_delegate]`, so the check reports `UseDelegate<GreeterTable>` as no provider of
//! `Greeter`. An `open` statement dispatches on a component's parameters too, so no `help` offers
//! one.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E115

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) {}
}

pub struct App;

delegate_components! {
    App {
        GreeterComponent: UseDelegate<new GreeterTable {
            App: GreetHello,
        }>,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
