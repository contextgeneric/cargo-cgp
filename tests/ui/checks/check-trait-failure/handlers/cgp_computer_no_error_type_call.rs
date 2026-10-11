//! `try_compute` on a context wiring a `#[cgp_computer]` provider and no error type.
//!
//! Beside the real `[CGP-E107]` on `ErrorTypeProviderComponent`, the tool reports a second root cause,
//! `[CGP-E110] provider `Add` does not contain any delegate entry for `ComputerComponent``, which is
//! false: the call-site anchor types the input `(_, _)`, so `Add`'s own impl does not match.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E107 CGP-E110

use core::marker::PhantomData;

use cgp::extra::handler::CanTryCompute;
use cgp::prelude::*;

#[cgp_computer]
fn add(a: u64, b: u64) -> u64 {
    a + b
}

pub struct App;

delegate_components! {
    App {
        TryComputerComponent: Add,
    }
}

fn main() {
    let _ = App.try_compute(PhantomData::<()>, (1, 2));
}
