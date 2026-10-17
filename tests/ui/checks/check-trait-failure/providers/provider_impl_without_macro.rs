//! A provider-trait impl written without `#[cgp_provider]`.
//!
//! The provider has no `IsProviderFor` impl, which the provider trait requires as a supertrait, so
//! the compiler rejects the impl on that bound and lists every unrelated implementor of
//! `IsProviderFor` beside it. The tool recognizes the impl structurally, words it as `[CGP-E039]`
//! with a `help` to add `#[cgp_provider]` or write the provider with `#[cgp_impl]`, and drops the
//! implementor list.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E039

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

pub struct GreetHello;

impl<Context> Greeter<Context> for GreetHello {
    fn greet(_context: &Context) -> String {
        "hello".to_owned()
    }
}

fn main() {}
