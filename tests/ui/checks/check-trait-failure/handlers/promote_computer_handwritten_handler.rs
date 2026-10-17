//! `PromoteComputer<Double>` wired as a handler for a hand-written `Computer` provider.
//!
//! `[CGP-E115]` says `Double` does not implement `TryComputer`, and its `help` names the adapter
//! that makes it one, `Promote<Double>`, with the `use` it needs, which the solver confirms.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E115

use core::marker::PhantomData;

use cgp::core::error::ErrorTypeProviderComponent;
use cgp::prelude::*;

#[cgp_new_provider]
impl<Context, Code> Computer<Context, Code, u64> for Double {
    type Output = u64;

    fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
        input * 2
    }
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        HandlerComponent: PromoteComputer<Double>,
    }
}

check_components! {
    App {
        HandlerComponent: ((), u64),
    }
}

fn main() {}
