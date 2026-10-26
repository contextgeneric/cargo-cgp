//! A `#[cgp_fn]` body calls `self.describe()` without `#[uses]`, where two CGP traits declare a
//! `describe` method: the `#[cgp_fn]` trait `Describe`, imported, and the consumer trait
//! `CanDescribe` in another module.
//!
//! The tool reports `[CGP-E030]` for `Describe`, the trait rustc's notes point at, rather than the
//! first candidate by name.
//!
//! CGP error class: undeclared dependency (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/undeclared-dependency.md).

//@quality: acceptable
//@codes: CGP-E030

use cgp::prelude::*;

pub mod named {
    use cgp::prelude::*;

    #[cgp_fn]
    pub fn describe(&self, #[implicit] name: &str) -> String {
        name.to_owned()
    }
}

pub mod component {
    use cgp::prelude::*;

    #[cgp_component(Describer)]
    pub trait CanDescribe {
        fn describe(&self) -> String;
    }
}

use named::Describe;

#[cgp_fn]
pub fn greet(&self) -> String {
    self.describe()
}

fn main() {}
