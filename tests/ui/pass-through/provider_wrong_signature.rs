//! A provider method whose signature differs from the provider trait's.
//!
//! The error is plain Rust: the method's type is incompatible with the trait (`E0053`), and the
//! tool leaves it alone. Its text still differs from plain `cargo check`, a documented limitation:
//! the injected `--verbose` prints the generated `__Context__` in the signature note where rustc
//! prints `_` (cgp-knowledge-base/cargo-cgp/implementation/driver/program.md, "Un-eliding the diagnostic").
//!
//! CGP error class: none; plain Rust the tool passes through.

//@rewrites-falsely: the injected `--verbose` prints the generated `__Context__` where rustc prints `_` (a documented limitation)

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
