//! The attribute `[CGP-E070]` offers for a provider header whose derived component name is not
//! its component's, as the fix an editor or `cargo fix` applies.
//!
//! The macro derives the component from the trait name the header writes, so a trait imported
//! under another name, `impl G`, names no component, and the fix binds the real one explicitly.
//! It adds the component to the header's attribute: `: Key` after the provider of
//! `#[cgp_impl(...)]`, and `(Key)` after `#[cgp_provider]`. A key the module does not import is
//! imported by the same fix, with the `use` line a missing component's import inserts, as in
//! `renamed_without_key`. A last, unchanged part spans the trait's name, where the error points,
//! so an editor offers the fix there. The fix is machine-applicable when the header writes an
//! argument for each of the key's parameters, and only maybe correct when it does not, as for
//! `AreaCalculatorComponent<Shape>` under `impl AreaCalculator`, where the parameter's name stands
//! in for the argument.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).

//@quality: acceptable
//@codes: CGP-E070
//@json

pub mod components {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    #[cgp_component {
        name: AreaCalculatorComponent<Shape>,
        provider: AreaCalculator,
    }]
    pub trait CanCalculateArea<Shape> {
        fn area(&self, shape: &Shape) -> f64;
    }
}

pub mod renamed_without_key {
    use cgp::prelude::*;

    use crate::components::Greeter as G;

    #[cgp_impl(new GreetHello)]
    impl G {
        fn greet(&self) -> String {
            "Hello".to_owned()
        }
    }
}

pub mod renamed_raw_provider {
    use cgp::prelude::*;

    use crate::components::{Greeter as G, GreeterComponent};

    pub struct GreetRaw;

    #[cgp_provider]
    impl<Context> G<Context> for GreetRaw {
        fn greet(_context: &Context) -> String {
            "Hi".to_owned()
        }
    }
}

pub mod parameterized {
    use cgp::prelude::*;

    use crate::components::{AreaCalculator, AreaCalculatorComponent};

    pub struct Square(pub f64);

    #[cgp_impl(new SquareArea)]
    impl AreaCalculator<Square> {
        fn area(&self, shape: &Square) -> f64 {
            shape.0 * shape.0
        }
    }

    #[cgp_impl(new AnyArea)]
    impl AreaCalculator {
        fn area(&self, _shape: &u32) -> f64 {
            1.0
        }
    }
}

fn main() {}
