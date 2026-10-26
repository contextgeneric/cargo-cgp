//! A provider of a component that requires `HasName` of every context (`#[extend(HasName)]`) and
//! `T: Display` of its parameter, declaring neither.
//!
//! The tool reports `[CGP-E036]` for the context's requirement alone, `HasName`, and leaves rustc's
//! `E0277` on `T: Display`, which `#[uses]` does not declare.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: acceptable
//@codes: CGP-E036

use core::fmt::Display;

use cgp::prelude::*;

#[cgp_auto_getter]
pub trait HasName {
    fn name(&self) -> &str;
}

#[cgp_component(Shower)]
#[extend(HasName)]
pub trait CanShow<T: Display> {
    fn show(&self, value: &T) -> String;
}

#[cgp_impl(new ShowPlain)]
impl<T> Shower<T> {
    fn show(&self, _value: &T) -> String {
        String::new()
    }
}

fn main() {}
