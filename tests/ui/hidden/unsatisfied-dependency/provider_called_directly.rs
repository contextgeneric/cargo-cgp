//! `PipeHandlers::<Product![…]>::compute(…)` called directly over stages returning `Result`.
//!
//! The provider-call anchor seeds the pipeline's `Computer` from the call, and the walk reaches the
//! second `Increment`, given the first one's `Result<u8, &str>` output where it takes `u8`:
//! ``[CGP-E127] `Increment` implements `Computer` only with `Input = u8`, not `Input = Result<u8, &str>` ``.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E002 CGP-E102 CGP-E127

use cgp::extra::handler::{Computer, PipeHandlers};
use cgp::prelude::*;

#[cgp_computer]
pub fn increment(value: u8) -> Result<u8, &'static str> {
    value.checked_add(1).ok_or("overflow")
}

fn main() {
    // `Increment` produces `Result<u8, _>` and the next `Increment` wants a `u8`, so plain
    // composition does not type-check.
    let _ = PipeHandlers::<Product![Increment, Increment]>::compute(&(), PhantomData::<()>, 1u8);
}
