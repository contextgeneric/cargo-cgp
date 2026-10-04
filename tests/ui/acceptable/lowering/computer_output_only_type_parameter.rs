//! A `#[cgp_computer]` function whose type parameter `T` appears only in its
//! return type. The macro moves the function's generics onto the generated
//! `Computer` impl, whose input type is the parameter list, so `T` is constrained
//! by neither the trait arguments nor the self type, and the impl fails with
//! `E0207`. This is a known limitation of the macro: accepting such a function
//! would need the provider struct itself to be generic over `T`. The caret falls
//! on the `T` the user wrote.
//!
//! Error class: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unconstrained-generic.md.

use cgp::prelude::*;

#[cgp_computer]
fn parse<T: core::str::FromStr>(value: String) -> Option<T> {
    value.parse().ok()
}

fn main() {}
