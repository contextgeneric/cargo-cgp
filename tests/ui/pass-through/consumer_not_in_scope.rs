//! A consumer method called where its trait is not imported.
//!
//! The tool prints `struct lib::Person` where plain `cargo check` prints `struct Person`.
//!
//! CGP error class: none; a driver path-printing defect affecting untransformed diagnostics.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@rewrites-falsely: untransformed output prints full paths, because the driver does not set `trimmed_def_paths`

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
