//! A wiring key naming a component defined in another module of the crate, without its import.
//!
//! `GreeterComponent` is declared in `components`, but the wiring module does not import it, so the
//! key is an `E0425` "cannot find type". rustc suggests the right import beside a similarly named
//! `ComputerComponent`, a different component. The tool reshapes it into `[CGP-E041]` with only the
//! import, `use crate::components::GreeterComponent;`.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: CGP-E041

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
