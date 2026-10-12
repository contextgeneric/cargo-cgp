//! `Add::try_compute(&App, …)` on a `#[cgp_computer]` provider, with no error type on `App`.
//!
//! An uncoded `E0277` "`App: ErrorTypeProvider<App>`", with rustc's implementor list; the missing
//! `ErrorTypeProviderComponent` wiring is never stated. rustc reports it twice on the call, and the
//! tool merges the repeat into the first block.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

//! `docs/reference/macros/cgp_computer.md`, *Common Mistakes*: the fallible members need an error
//! type on the context, while `compute` does not.

use core::marker::PhantomData;

use cgp::extra::handler::{Computer, TryComputer};
use cgp::prelude::*;

#[cgp_computer]
fn add(a: u64, b: u64) -> u64 {
    a + b
}

pub struct App;

fn main() {
    let _ = Add::compute(&App, PhantomData::<()>, (1, 2));
    let _ = Add::try_compute(&App, PhantomData::<()>, (1, 2));
}
