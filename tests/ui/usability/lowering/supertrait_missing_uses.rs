//! A provider for a component with `#[extend(HasName)]` that omits `#[uses(HasName)]`.
//!
//! The `E0277` on `__Context__: HasName` carries the getter's help "`#[derive(HasField)]` is required
//! to access field `name` on `__Context__`", which is wrong advice; the fix is `#[uses(HasName)]`.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_component(Greeter)]
#[extend(HasName)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) {}
}

#[derive(HasField)]
pub struct Person {
    pub name: String,
}

delegate_components! {
    Person {
        GreeterComponent: GreetHello,
    }
}

fn main() {
    Person { name: String::new() }.greet();
}
