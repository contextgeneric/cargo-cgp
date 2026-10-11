//! `PromoteRef<Double>` for a `Double` that takes its input by value.
//!
//! A raw `E0271` on `<Double as Computer<App, (), &u64>>::Output == _`, passed through except for
//! the renamed notes.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

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

use cgp::extra::handler::PromoteRef;

delegate_components! {
    App {
        ComputerRefComponent: PromoteRef<Double>,
    }
}

check_components! {
    App {
        ComputerRefComponent: ((), u64),
    }
}

fn main() {}
