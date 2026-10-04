//! A `#[cgp_computer]` function returning `Result<u64, String>`, called through
//! `try_compute` on a context whose abstract error type is `()` rather than
//! `String`. The fallible promotion passes the function's `Err` through
//! unconverted, so it requires the context's error type to equal the function's,
//! and the call fails with `E0271`. The note names both `Result` types and points
//! at the function's return type, so the mismatch is stated plainly.
//!
//! Error class: an associated-type mismatch,
//! https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/error_codes/e0271.md.

use cgp::core::error::ErrorTypeProviderComponent;
use cgp::extra::handler::TryComputer;
use cgp::prelude::*;

#[cgp_computer]
fn checked_add(a: u64, b: u64) -> Result<u64, String> {
    a.checked_add(b).ok_or_else(|| "overflow".to_owned())
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<()>,
    }
}

fn main() {
    let _ = CheckedAdd::try_compute(&App, PhantomData::<()>, (1, 2));
}
