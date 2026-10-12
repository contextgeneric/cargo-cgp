//! A provider that omits `#[uses(HasName)]` and also holds an unrelated error in its body.
//!
//! The missing declaration fails with `E0277` on `__Context__: HasName` at the `#[cgp_impl]`
//! attribute, which the tool reshapes into `[CGP-E036]`, and the body passes an `Rc` to a function
//! that requires `Send`, a plain Rust `E0277` the reshape leaves as rustc wrote it: an error inside
//! a method body is about the body's own code.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: acceptable
//@codes: CGP-E036

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
