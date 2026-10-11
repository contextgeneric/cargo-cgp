//! A provider method whose signature differs from the provider trait's.
//!
//! The tool prints `fn(&__Context__, u32)` where plain `cargo check` prints `fn(&_, u32)`, an effect
//! of the injected `--verbose`.
//!
//! CGP error class: none; a driver path-printing defect affecting untransformed diagnostics.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@rewrites-falsely: untransformed output prints full paths, because the driver does not set `trimmed_def_paths`

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self, times: u32) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self, times: u64) -> String {
        format!("{times}")
    }
}

pub struct Person;

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}

fn main() {
    Person.greet(1);
}
