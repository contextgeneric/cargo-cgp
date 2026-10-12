//! Two `check_components!` tables whose `#[check_trait]` attributes give one name.
//!
//! The second definition of `CheckApp` is an `E0428`, named as rustc reports it.
//!
//! CGP error class: conflicting wiring (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md).

//@quality: usability
//@codes: none

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
    #[check_trait(CheckApp)]
    App {
        GreeterComponent,
    }
}

check_components! {
    #[check_trait(CheckApp)]
    App {
        GreeterComponent,
    }
}

fn main() {}
