//! A provider body names `Inner::Error` for an inner provider parameter, not for the context.
//!
//! The tool leaves rustc's `E0220` as written: `#[use_type]` imports an abstract type of the context,
//! so it cannot be the fix for a type sought on `Inner`.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: acceptable
//@codes: none

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

#[cgp_impl(new GreetWith<Inner>)]
impl<Inner> Greeter {
    fn greet(&self) -> String {
        let _error: Option<Inner::Error> = None;
        String::new()
    }
}

fn main() {}
