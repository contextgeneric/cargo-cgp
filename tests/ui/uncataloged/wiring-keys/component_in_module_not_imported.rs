//! A wiring key naming a component defined in another module of the crate, without its import.
//!
//! `GreeterComponent` is declared in `components`, but the wiring module does not import it, so the
//! key is an `E0425` "cannot find type". rustc suggests the right import,
//! `use crate::components::GreeterComponent;`, beside a similarly named `ComputerComponent`, which
//! is a different component.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

pub mod components {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self);
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self) {}
    }
}

pub mod wiring {
    use cgp::prelude::*;

    use crate::components::GreetHello;

    pub struct Person;

    delegate_components! {
        Person {
            GreeterComponent: GreetHello,
        }
    }
}

fn main() {}
