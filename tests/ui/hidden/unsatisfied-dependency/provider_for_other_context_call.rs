//! A provider written for one context, wired on another, and reached by a call.
//!
//! The resolver declines, so the call is a raw `E0599` that leaks `IsProviderFor` and its source path
//! in `cgp`.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

pub struct Person;
pub struct Robot;

#[cgp_impl(new GreetPerson)]
impl Greeter for Person {
    fn greet(&self) {}
}

delegate_components! {
    Robot {
        GreeterComponent: GreetPerson,
    }
}

fn main() {
    Robot.greet();
}
