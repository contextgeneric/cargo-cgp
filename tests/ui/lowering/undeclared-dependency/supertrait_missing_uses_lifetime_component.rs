//! A provider for a component with a lifetime parameter and `#[extend(HasName)]` that omits
//! `#[uses(HasName)]`.
//!
//! The component requires `HasName` of every context, so the provider trait carries the bound and
//! rustc reports `E0277` on `__Context__: HasName` at the `#[cgp_impl]` attribute. The provider
//! trait takes the component's lifetime ahead of the context, so a reader that took the context by
//! raw position would land on the lifetime; the tool reads it by type position and reshapes the
//! error into `[CGP-E036]`, as it does for a component without parameters.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_component(Greeter)]
#[extend(HasName)]
pub trait CanGreet<'a> {
    fn greet(&self, other: &'a str) -> String;
}

// MISTAKE: the provider does not declare the `HasName` the component requires.
#[cgp_impl(new GreetHello)]
impl<'a> Greeter<'a> {
    fn greet(&self, other: &'a str) -> String {
        other.to_owned()
    }
}

fn main() {}
