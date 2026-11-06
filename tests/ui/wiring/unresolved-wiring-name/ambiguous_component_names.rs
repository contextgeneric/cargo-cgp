//! Wiring keys that could name either of two components, so the tool names neither.
//!
//! Modules `a` and `c` each define a component named `GreeterComponent`, and modules `a` and `b`
//! each define a consumer trait named `CanGreet`, with the components `GreeterComponent` and
//! `WelcomerComponent`. None is imported at the crate root. `Person` wires the key
//! `GreeterComponent`, which either module's component could be, and `Robot` the key
//! `CanGreetComponent`, guessed from a consumer trait's name, which could mean either `CanGreet`.
//! The tool offers no import and no component for either, and leaves each key's `E0425` as rustc
//! wrote it, dropping only rustc's repeat of the second at the same key. The first suggests
//! importing each `GreeterComponent`, beside a similarly named `ComputerComponent`, and the second
//! suggests the similarly named trait `CanUseComponent` and a type parameter, none of which fits a
//! wiring key.
//!
//! CGP error class: unresolved wiring name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unresolved-wiring-name.md).
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md

//@quality: usability
//@codes: none

use cgp::prelude::*;

pub mod a {
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

pub mod b {
    use cgp::prelude::*;

    #[cgp_component(Welcomer)]
    pub trait CanGreet {
        fn greet(&self);
    }

    #[cgp_impl(new WelcomeHello)]
    impl Welcomer {
        fn greet(&self) {}
    }
}

pub mod c {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanHail {
        fn hail(&self);
    }
}

pub struct Person;

pub struct Robot;

delegate_components! {
    Person {
        GreeterComponent: a::GreetHello,
    }
}

delegate_components! {
    Robot {
        CanGreetComponent: b::WelcomeHello,
    }
}

fn main() {}
