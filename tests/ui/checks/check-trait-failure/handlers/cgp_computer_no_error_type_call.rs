//! `try_compute` on a context wiring a `#[cgp_computer]` provider and no error type.
//!
//! The one root cause is `[CGP-E113]` on `ErrorTypeProviderComponent`. The call-site anchor types
//! the input `(_, _)`, so `Add`'s own `Computer` impl does not match and the walk reaches its
//! delegation blanket; `Add` has its own impl of the trait, so the unmet `DelegateComponent` there
//! is a dead end rather than a missing dispatch entry, even though `Add` wires its promotions.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E113

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
