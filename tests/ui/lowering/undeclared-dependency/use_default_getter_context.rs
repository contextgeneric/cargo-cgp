//! A `UseDefault` provider whose default body calls a getter it does not declare in `#[uses]`.
//!
//! The component's `#[extend(HasName)]` requires `HasName` of every context, so rustc reports
//! `E0277` on `__Context__: HasName` at the `#[cgp_impl(UseDefault)]` attribute, leaking the
//! generated `__Context__`. The tool reshapes it into `[CGP-E036]`, naming `UseDefault` as the
//! provider, with `#[uses(HasName)]` in a `help`. The resolver skips the `NameGetterComponent`
//! marker, defined under the `#[cgp_getter]` attribute rustc points at, as a context candidate, so
//! it states no false context.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: acceptable
//@codes: CGP-E036

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
