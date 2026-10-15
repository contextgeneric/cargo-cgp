//! A pipeline stage taking `u64` after one returning `String`.
//!
//! The walk reaches `Double: Computer<App, (), String>`, which no impl of `Double` matches, since
//! `Double` takes only `u64`, and names that argument:
//! ``[CGP-E127] `Double` implements `Computer` only with `Input = u64`, not `Input = String` ``.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E127

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
