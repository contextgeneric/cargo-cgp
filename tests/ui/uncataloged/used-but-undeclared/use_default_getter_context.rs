//! A `UseDefault` provider whose default body calls a getter it does not declare in `#[uses]`.
//!
//! The tool keeps rustc's `E0277` that `__Context__: HasName` is not satisfied, required by the
//! `#[extend(HasName)]` bound on `Greeter`. Nothing names the fix, `#[uses(HasName)]` on the
//! provider, and the message leaks the generated `__Context__`. The resolver skips the
//! `NameGetterComponent` marker, defined under the `#[cgp_getter]` attribute rustc points at, as a
//! context candidate, so it states no false context.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

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
