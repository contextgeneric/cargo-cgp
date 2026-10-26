//! A provider body names `Self::Fault`, where the only abstract-type trait declaring `Fault` takes a
//! parameter of its own, `HasErrorTypeFor<Tag>`.
//!
//! The tool leaves rustc's `E0220` as written: `#[use_type(HasErrorTypeFor.Fault)]` would leave out
//! the trait's parameter, so the import is not one it can name.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: acceptable
//@codes: none

use cgp::prelude::*;

#[cgp_type]
pub trait HasErrorTypeFor<Tag> {
    type Fault;
}

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetHello)]
impl Greeter {
    fn greet(&self) -> String {
        let _fault: Option<Self::Fault> = None;
        String::new()
    }
}

fn main() {}
