//! Two `check_components!` tables for one context.
//!
//! `E0428` "`__CheckApp` is defined multiple times"; the `#[check_trait(Name)]` fix is not hinted.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

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

delegate_components! {
    App {
        GreeterComponent: GreetHello,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
