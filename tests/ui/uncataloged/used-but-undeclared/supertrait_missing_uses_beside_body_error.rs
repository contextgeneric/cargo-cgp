//! A provider that omits `#[uses(HasName)]` and also holds an unrelated error in its body.
//!
//! The missing declaration fails with `E0277` on `__Context__: HasName` at the `#[cgp_impl]`
//! attribute, and the body passes an `Rc` to a function that requires `Send`, a plain Rust
//! `E0277` that must keep its own diagnostic.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use std::rc::Rc;

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

fn require_send<T: Send>(_value: T) {}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) {
        require_send(Rc::new(1));
    }
}

fn main() {}
