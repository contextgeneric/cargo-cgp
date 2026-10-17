//! An `@`-path segment naming a component that is not in scope.
//!
//! The segment `GreeterComponent` is a component of another module, so it needs its import. The
//! compiler's suggestions include a similarly named component of another kind. The tool words the
//! segment as `[CGP-E041]`, the same as a component written as a key, with the import path.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: CGP-E041

use cgp::prelude::*;

mod parts {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self) -> String {
            "hello".into()
        }
    }
}

use parts::GreetHello;

pub struct App;

delegate_components! {
    App {
        @app.GreeterComponent: GreetHello,
    }
}

fn main() {}
