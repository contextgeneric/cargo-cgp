//! A provider for a component with `#[extend(HasName)]` that omits `#[uses(HasName)]`.
//!
//! The component requires `HasName` of every context, so the provider trait carries the bound and
//! rustc reports `E0277` on `__Context__: HasName` at the `#[cgp_impl]` attribute, with a help about
//! the `HasField` bound inside `HasName`. The tool reshapes it into `[CGP-E036]`, naming the
//! provider trait, the provider, and the requirement, with `#[uses(HasName)]` in a `help`.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.

//@quality: acceptable
//@codes: CGP-E036

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
