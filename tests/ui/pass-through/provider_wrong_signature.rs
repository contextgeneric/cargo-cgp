//! A provider method whose signature differs from the provider trait's.
//!
//! The error is plain Rust: the method's type is incompatible with the trait (`E0053`). The tool
//! must print what plain `cargo check` prints; under the injected `--verbose`, rustc prints the
//! generated `__Context__` in the signature note where plain `cargo check` prints `_`.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.

//@rewrites-falsely: the injected `--verbose` prints the generated `__Context__` where rustc prints `_`

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
