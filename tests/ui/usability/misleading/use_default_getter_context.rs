//! A `UseDefault` provider whose default body calls a getter it does not declare in `#[uses]`.
//!
//! The headline names `NameGetterComponent` as the context, the same misnamed context as an unwired
//! getter call; the fix is `#[uses(HasName)]` on the provider.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use cgp::core::component::UseDefault;
use cgp::prelude::*;

#[cgp_getter]
pub trait HasName {
    fn name(&self) -> &str {
        "John"
    }
}

#[cgp_component(Greeter)]
#[extend(HasName)]
pub trait CanGreet {
    fn greet(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}

#[cgp_impl(UseDefault)]
impl Greeter {}

fn main() {}
