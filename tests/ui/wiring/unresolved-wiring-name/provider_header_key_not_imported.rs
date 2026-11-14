//! Provider headers that import a component's provider trait from another module but not its key.
//!
//! `#[cgp_impl]` and `#[cgp_provider]` name the component after the provider trait, as
//! `GreeterComponent`, so the module must import that key beside `Greeter`. Without it, each header
//! is an `E0425` "cannot find type `GreeterComponent`" on the trait's name, which the tool reshapes
//! into `[CGP-E041]` with the import, `use crate::components::GreeterComponent;`.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: CGP-E041

pub mod components {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }
}

pub mod providers {
    use cgp::prelude::*;

    use crate::components::Greeter;

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self) -> String {
            "Hello".to_owned()
        }
    }

    pub struct GreetRaw;

    #[cgp_provider]
    impl<Context> Greeter<Context> for GreetRaw {
        fn greet(_context: &Context) -> String {
            "Hi".to_owned()
        }
    }
}

fn main() {}
