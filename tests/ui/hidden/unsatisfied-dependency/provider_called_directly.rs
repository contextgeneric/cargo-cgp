//! `PipeHandlers::<Product![…]>::compute(…)` called directly over stages returning `Result`.
//!
//! A raw `E0599` naming the unmet `IsProviderFor` bound and pointing into `cgp-handler`; nothing says
//! the stages' `Result` outputs do not compose without the monadic pipeline.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

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
