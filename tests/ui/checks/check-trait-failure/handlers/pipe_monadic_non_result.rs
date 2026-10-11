//! `PipeMonadic<ErrMonadic, …>` over stages that do not return `Result`.
//!
//! `[CGP-E002]` on `BindErr<IdentMonadic, Double>` with a help quoting `cgp-monad` source; nothing says
//! the stages must return `Result`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E002

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
