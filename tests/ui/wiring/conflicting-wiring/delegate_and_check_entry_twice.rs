//! A `delegate_and_check_components!` table listing one entry twice.
//!
//! The repeated entry wires its key twice, an `E0119` on the `DelegateComponent` impls that becomes
//! `[CGP-E010]`, and checks it twice, an `E0119` on the two identical check impls. The second is
//! the same mistake, so the tool reports the wiring conflict alone.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).

//@quality: acceptable
//@codes: CGP-E010

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

pub struct App;

// MISTAKE: the table lists `GreeterComponent` twice.
delegate_and_check_components! {
    App {
        GreeterComponent: GreetHello,
        GreeterComponent: GreetHello,
    }
}

fn main() {}
