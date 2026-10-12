//! Two `check_components!` tables for one context.
//!
//! The `E0428` on the derived `__CheckApp` becomes `[CGP-E016]`, naming the context and the fix.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).
//!
//! The two carets are relabelled as the first and the second table, in place of rustc's labels.

//@quality: acceptable
//@codes: CGP-E016

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
