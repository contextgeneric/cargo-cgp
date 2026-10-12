//! Two `#[cgp_auto_dispatch]` traits in one module that share two method names, `area` and
//! `perimeter`.
//!
//! Each shared method makes the macro define its `__compute_{method}__` helper and its
//! `Compute{Method}` computer twice, so rustc reports four `E0428`s, two per method, naming the
//! generated items, and nothing says the two traits share the methods.
//!
//! Error class: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md, with the name clash itself
//! under https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/error_codes/e0428.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_auto_dispatch]
pub trait HasArea {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
}

#[cgp_auto_dispatch]
pub trait HasScaledArea {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
}

fn main() {}
