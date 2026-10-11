//! A provider-trait impl written without `#[cgp_provider]`.
//!
//! A `[CGP-E002]` headline, then rustc's list of every `IsProviderFor` implementor in scope; the fix,
//! `#[cgp_provider]` or `#[cgp_impl]`, is not hinted.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

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
