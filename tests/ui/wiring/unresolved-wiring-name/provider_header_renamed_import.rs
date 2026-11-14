//! Provider headers that name a provider trait imported under another name.
//!
//! With `use crate::components::{Greeter as G, GreeterComponent};`, the header `impl G` makes the
//! macro name the component `GComponent`, which nothing declares, so rustc reports "cannot find
//! type `GComponent`" with no suggestion and calls the correct `GreeterComponent` import unused.
//! The tool reshapes each into `[CGP-E070]`, naming the trait's real component and the attribute
//! form that names it.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: CGP-E070

pub mod components {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }
}

pub mod providers {
    use cgp::prelude::*;

    use crate::components::{Greeter as G, GreeterComponent};

    #[cgp_impl(new GreetHello)]
    impl G {
        fn greet(&self) -> String {
            "Hello".to_owned()
        }
    }

    pub struct GreetRaw;

    #[cgp_provider]
    impl<Context> G<Context> for GreetRaw {
        fn greet(_context: &Context) -> String {
            "Hi".to_owned()
        }
    }
}

fn main() {}
