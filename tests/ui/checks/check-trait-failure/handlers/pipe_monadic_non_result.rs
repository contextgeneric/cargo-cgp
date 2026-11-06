//! `PipeMonadic<ErrMonadic, …>` over stages that do not return `Result`.
//!
//! The error monad binds each later stage with `BindErr`, which takes a `Result` input, so the walk
//! reaches `BindErr<IdentMonadic, Double>` given the first stage's `u64`:
//! ``[CGP-E127] `BindErr<IdentMonadic, Double>` implements `Computer` only with `Input = Result<_, _>`, not `Input = u64` ``.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E127

use core::marker::PhantomData;

use cgp::prelude::*;

#[cgp_new_provider]
impl<Context, Code> Computer<Context, Code, u64> for Double {
    type Output = u64;

    fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> u64 {
        input * 2
    }
}

pub struct App;

use cgp::extra::monad::monadic::err::ErrMonadic;
use cgp::extra::monad::providers::PipeMonadic;

delegate_components! {
    App {
        ComputerComponent: PipeMonadic<ErrMonadic, Product![Double, Double]>,
    }
}

check_components! {
    App {
        ComputerComponent: ((), u64),
    }
}

fn main() {}
