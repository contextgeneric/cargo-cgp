//! A pipeline stage taking `u64` after one returning `String`.
//!
//! A `[CGP-E002]` headline with rustc's "expected `u64`, found `String`" help, no tree, and no mention
//! of which stages disagree.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use core::marker::PhantomData;

use cgp::extra::handler::PipeHandlers;
use cgp::prelude::*;

#[cgp_new_provider]
impl<Context, Code> Computer<Context, Code, u64> for ToText {
    type Output = String;

    fn compute(_context: &Context, _code: PhantomData<Code>, input: u64) -> String {
        input.to_string()
    }
}

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
        ComputerComponent: PipeHandlers<Product![ToText, Double]>,
    }
}

check_components! {
    App {
        ComputerComponent: ((), u64),
    }
}

fn main() {}
