//! A provider header that names its provider trait through a module re-exporting the trait alone.
//!
//! The header `impl crate::facade::Greeter` names the component by the same path,
//! `crate::facade::GreeterComponent`, but `facade` re-exports `Greeter` without its key, so the
//! header is an `E0425` "cannot find type `GreeterComponent` in module `crate::facade`". Importing
//! the key does not fix a header that names a path; the tool reshapes it into `[CGP-E041]` naming
//! the module, with the trait's path through the module that defines its key.
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

pub mod facade {
    pub use crate::components::Greeter;
}

pub mod providers {
    use cgp::prelude::*;

    #[cgp_impl(new GreetHello)]
    impl crate::facade::Greeter {
        fn greet(&self) -> String {
            "Hello".to_owned()
        }
    }
}

fn main() {}
