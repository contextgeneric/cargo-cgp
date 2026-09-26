//! Acceptable failure: an `open` key with a generic first segment overlapping a one-segment key.
//! `@ComputerComponent.<Code> Code.u64` dispatches on the input, the component's second parameter,
//! for every code, so at `Code = Eval` it covers a key that `@ComputerComponent.Eval` already
//! claims. Each path entry lowers to a `DelegateComponent<PathCons<..>>` with a wildcard tail, and
//! the generic segment makes the two impls overlap with the coherence error E0119.
//!
//! The tool drops the redundant `IsProviderFor` half and rewrites the `DelegateComponent` half to
//! `[CGP-E005] `App` cannot wire `@ComputerComponent.*.u64.*` that is already set through
//! `@ComputerComponent.Eval.*``: the generic segment renders as `*` in its own position, so the
//! concrete `u64` segment after it still shows which key the entry names.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E005).

use cgp::extra::handler::{Computer, ComputerComponent};
use cgp::prelude::*;

/// The code the one-segment key names.
pub struct Eval;

#[cgp_impl(new Double)]
impl<Code> Computer<Code, u64> {
    type Output = u64;

    fn compute(&self, _code: PhantomData<Code>, input: u64) -> u64 {
        input * 2
    }
}

#[cgp_impl(new Show)]
impl<Code, Input: core::fmt::Debug> Computer<Code, Input> {
    type Output = String;

    fn compute(&self, _code: PhantomData<Code>, input: Input) -> String {
        format!("{input:?}")
    }
}

pub struct App;

delegate_components! {
    App {
        open ComputerComponent;

        @ComputerComponent.Eval: Show,
        @ComputerComponent.<Code> Code.u64: Double,
    }
}

fn main() {}
