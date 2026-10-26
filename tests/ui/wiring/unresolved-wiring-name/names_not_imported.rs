//! Wiring names and a `#[cgp_impl]` provider that are declared, in another module or in a
//! dependency, but not imported where they are written.
//!
//! `GreetHello` and `GreetBye` are declared in `providers`, `Square` in `shapes`, and `RaiseFrom` in
//! `cgp::extra::error`, and none is imported at the crate root. rustc reports an `E0425` for each
//! name, with a suggestion to import it where it finds one. The tool rewords them as names that are
//! not declared: `[CGP-E043]` for the two values, dropping the import suggestion, `[CGP-E038]` for
//! the `#[cgp_impl(GreetBye)]` header, with a help to declare a second `GreetBye`, and `[CGP-E044]`
//! for the path segment. Each statement is false, since every name is declared.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: usability
//@codes: CGP-E038 CGP-E043 CGP-E044

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
