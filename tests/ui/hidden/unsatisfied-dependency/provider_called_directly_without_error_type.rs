//! `Add::try_compute(&App, …)` on a `#[cgp_computer]` provider, with no error type on `App`.
//!
//! The provider-call anchor seeds `Add: TryComputer<App, (), (_, _)>` from the call, and the walk
//! reaches `App: HasErrorType` through `Add` and its `Promote<Add>` promotion:
//! ``[CGP-E002] missing wiring for `ErrorTypeProviderComponent` on `App` ``, with the wiring
//! `help`. rustc reports it twice on the call, and the repeat resolves to the same cause and is
//! dropped.
//!
//! The knowledge base's `cgp-knowledge-base/cgp/reference/macros/cgp_computer.md`, *Common
//! Mistakes*, states the rule: the fallible members need an error type on the context, while
//! `compute` does not.
//!
//! CGP error class: unsatisfied dependency, hidden (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@quality: acceptable
//@codes: CGP-E002 CGP-E101 CGP-E102 CGP-E113

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
