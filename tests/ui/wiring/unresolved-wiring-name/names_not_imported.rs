//! Wiring names and a `#[cgp_impl]` provider that are declared, in another module or in a
//! dependency, but not imported where they are written.
//!
//! `GreetHello` and `GreetBye` are declared in `providers`, `Square` in `shapes`, and `RaiseFrom` in
//! `cgp::extra::error`, and none is imported at the crate root, where `GreetBye` is also wired as
//! `Robot`'s provider. rustc reports an `E0425` for each name, with a suggestion to import it where
//! it finds one, which it does for the crate's own types.
//! Each name is declared, so the tool does not call any of them undeclared: it leaves every `E0425`
//! as rustc wrote it, the import suggestions included.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: none

use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

pub mod providers {
    use cgp::prelude::*;

    use super::{Greeter, GreeterComponent};

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self) {}
    }

    pub struct GreetBye;
}

pub mod shapes {
    pub struct Square;
}

#[cgp_impl(GreetBye)]
impl Greeter {
    fn greet(&self) {}
}

pub struct Person;

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}

pub struct Robot;

delegate_components! {
    Robot {
        GreeterComponent: GreetBye,
    }
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        ErrorRaiserComponent: RaiseFrom,
        GreeterComponent => @app.Square,
        @app.Square: providers::GreetHello,
    }
}

fn main() {}
