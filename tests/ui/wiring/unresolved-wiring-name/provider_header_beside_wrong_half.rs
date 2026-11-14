//! A provider header missing its key beside headers that name the wrong trait.
//!
//! A header naming the consumer trait (`impl CanGreet`) or a non-CGP trait (`impl Wave`) also fails
//! with an `E0425` for the `…Component` it derives, but the mistake there is the trait, reported as
//! `[CGP-E032]` and `[CGP-E033]`, with that `E0425` dropped. Only the header naming the provider
//! trait without its key is `[CGP-E041]`.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md),
//! and wrong component half (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/wrong-component-half.md).

//@quality: acceptable
//@codes: CGP-E032 CGP-E033 CGP-E041

pub mod components {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self) -> String;
    }

    pub trait Wave {
        fn wave(&self) -> String;
    }
}

pub mod providers {
    use cgp::prelude::*;

    use crate::components::{CanGreet, Greeter, Wave};

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self) -> String {
            "Hello".to_owned()
        }
    }

    #[cgp_impl(new GreetConsumer)]
    impl CanGreet {
        fn greet(&self) -> String {
            "Hi".to_owned()
        }
    }

    #[cgp_impl(new WaveHand)]
    impl Wave {
        fn wave(&self) -> String {
            "wave".to_owned()
        }
    }
}

fn main() {}
