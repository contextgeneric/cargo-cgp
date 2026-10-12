//! The same component listed twice in one `check_components!` table.
//!
//! The `E0119` between the two identical check impls becomes `[CGP-E015]`, with a removal help.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).
//!
//! The two carets cover the table from its start, where rustc puts each check impl's span.

//@quality: acceptable
//@codes: CGP-E015

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
        GreeterComponent,
    }
}

fn main() {}
