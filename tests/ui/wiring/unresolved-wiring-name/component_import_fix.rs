//! The import `[CGP-E041]` offers for a component key that is not in scope, as the fix an editor
//! or `cargo fix` applies.
//!
//! A provider header and a wiring key each name a component the module does not import. The fix
//! inserts the `use` line where rustc puts its own import suggestion: before the module's first
//! `use` line, or at the start of a module that has none, followed by a blank line. A second,
//! unchanged part spans the name the error points at, so an editor offers the fix there. The fix
//! is machine-applicable when one public path reaches the key, as for a key of this crate,
//! `cgp`'s `ErrorTypeProviderComponent`, and `cgp_test_component_library`'s `WaverComponent`,
//! and only maybe correct when two do, as for `cgp_test_crate_a`'s `GreeterComponent`, which
//! `cgp_test_component_library` re-exports.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: CGP-E041
//@json
//@aux-build: cgp-test-crate-a
//@aux-build: cgp-test-component-library

pub mod components {
    use cgp::prelude::*;

    #[cgp_component(Welcomer)]
    pub trait CanWelcome {
        fn welcome(&self) -> String;
    }
}

pub mod providers {
    use cgp::prelude::*;

    use crate::components::Welcomer;

    #[cgp_impl(new WelcomeHello)]
    impl Welcomer {
        fn welcome(&self) -> String {
            "Hello".to_owned()
        }
    }
}

pub mod contexts {
    pub struct Person;

    cgp::prelude::delegate_components! {
        Person {
            WelcomerComponent: crate::providers::WelcomeHello,
        }
    }
}

pub mod dependency {
    use cgp::core::error::ErrorTypeProvider;
    use cgp::prelude::*;
    use cgp_test_component_library::direct::Waver;
    use cgp_test_crate_a::Greeter;

    #[cgp_impl(new UseStringError)]
    impl ErrorTypeProvider {
        type Error = String;
    }

    #[cgp_impl(new WaveHand)]
    impl Waver {
        fn wave(&self) -> String {
            "wave".to_owned()
        }
    }

    #[cgp_impl(new GreetFromLibrary)]
    impl Greeter {
        fn greet(&self) -> String {
            "Hello".to_owned()
        }
    }
}

fn main() {}
