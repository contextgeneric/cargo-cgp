//! A consumer method called where its trait is not imported.
//!
//! The error is plain Rust: the trait that provides `greet` is not in scope. The tool must print
//! what plain `cargo check` prints, short paths included (`struct Person`, not `lib::Person`).
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.

mod lib {
    use cgp::prelude::*;

    #[cgp_component(Greeter)]
    pub trait CanGreet {
        fn greet(&self);
    }

    #[cgp_impl(new GreetHello)]
    impl Greeter {
        fn greet(&self) {}
    }

    pub struct Person;

    delegate_components! {
        Person {
            GreeterComponent: GreetHello,
        }
    }
}

fn main() {
    lib::Person.greet();
}
