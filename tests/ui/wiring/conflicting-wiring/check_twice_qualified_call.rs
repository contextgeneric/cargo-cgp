//! Two `check_components!` tables for one context, the second called by its path.
//!
//! The compiler records the second table's macro as `cgp::prelude::check_components`, and the tool
//! reads a macro by its path's last segment, so the `E0428` on the derived `__CheckApp` becomes
//! `[CGP-E016]`, as for two bare calls (`check_twice_same_context`).
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

cgp::prelude::check_components! {
    App {
        GreeterComponent,
    }
}

fn main() {}
