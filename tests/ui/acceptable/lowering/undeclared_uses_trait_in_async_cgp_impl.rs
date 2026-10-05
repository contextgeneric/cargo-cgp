//! The async form of [`undeclared_uses_trait_in_cgp_impl`](undeclared_uses_trait_in_cgp_impl.rs):
//! a `#[cgp_impl]` provider whose `async fn` body calls a trait method without declaring the trait
//! via `#[uses]`.
//!
//! An `async fn` body does not see its parameters directly: lowering moves each one into the future
//! with an untyped `let __context__ = __context__;`, so the failing call's receiver resolves to that
//! fresh binding rather than to the parameter declared `&__Context__`. Raw rustc reports the same
//! `E0599` about `&__Context__` as the sync case. The tool follows the forwarding `let` back to the
//! parameter and reshapes it into the same `[CGP-E012]` error with the `#[uses(PersonName)]` help.
//!
//! CGP error class:
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md.

use cgp::prelude::*;

#[cgp_fn]
fn person_name(&self, #[implicit] name: &str) -> String {
    name.to_owned()
}

#[cgp_component(Greeter)]
#[async_trait]
pub trait CanGreet {
    async fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    async fn greet(&self) -> String {
        format!("Hello, {}!", self.person_name())
    }
}

fn main() {}
