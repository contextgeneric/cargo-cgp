//! A `check_components!` table and a `delegate_and_check_components!` table whose
//! `#[check_trait]` attributes give one name.
//!
//! The second definition of `CheckApp` is an `E0428`. The tables come from different macros, so
//! the tool names each of them, and its fix renames one check trait.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).

//@quality: usability
//@codes: CGP-E016

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_component(Namer)]
pub trait CanName {
    fn name(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        "hello".to_owned()
    }
}

#[cgp_impl(new NameFixed)]
impl Namer {
    fn name(&self) -> String {
        "app".to_owned()
    }
}

pub struct App;

delegate_components! {
    App {
        GreeterComponent: GreetHello,
    }
}

check_components! {
    #[check_trait(CheckApp)]
    App {
        GreeterComponent,
    }
}

// MISTAKE: the second table's check trait takes the name the first one gave.
delegate_and_check_components! {
    #[check_trait(CheckApp)]
    App {
        NamerComponent: NameFixed,
    }
}

fn main() {}
