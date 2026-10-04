//! Upstream crate for the cross-crate fixtures of `#[cgp_auto_dispatch]`,
//! `#[cgp_computer]`, and `#[cgp_producer]`.
//!
//! It defines dispatch traits but no type implementing them, so every payload and
//! enum lives downstream: the generated enum-level blanket impl, the per-variant
//! `Compute{Method}` providers, and the helper functions they call all have to
//! work from a crate other than the one that declared the trait. It also defines
//! a computer and a producer for downstream code to call and wire.
//!
//! A fixture pulls it in with a `//@aux-build: cgp-test-dispatch-traits` header
//! directive; `cgp-test-dispatch-shapes` builds on it.

use cgp::prelude::*;

/// A dispatch trait covering the receiver forms: a `&self` reader, a `&mut self`
/// mutator taking an argument, a by-value method, and a method borrowing through
/// an elided lifetime.
#[cgp_auto_dispatch]
pub trait HasShape {
    fn area(&self) -> f64;

    fn scale(&mut self, factor: f64);

    fn into_name(self) -> &'static str;

    fn label(&self, prefix: &str) -> String;
}

/// An async dispatch trait, stacked with `#[async_trait]`.
#[cgp_auto_dispatch]
#[async_trait]
pub trait CanDescribe {
    async fn describe(&self) -> String;
}

/// A synchronous computer, named `Double`.
#[cgp_computer]
pub fn double(value: f64) -> f64 {
    value * 2.0
}

/// A fallible computer, named `CheckedHalve`.
#[cgp_computer]
pub fn checked_halve(value: u64) -> Result<u64, String> {
    if value % 2 == 0 {
        Ok(value / 2)
    } else {
        Err(format!("{value} is odd"))
    }
}

/// A producer, named `UnitScale`.
#[cgp_producer]
pub fn unit_scale() -> f64 {
    1.0
}
