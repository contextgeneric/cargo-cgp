//! A provider written for another context and another input, wired and checked.
//!
//! `Describe` implements `Computer` for `Person` with input `u8`, and `Robot` wires it and checks it
//! at input `u16`, so two arguments differ from every impl the provider has. No single argument is
//! the one to change, so the tool does not name a provider written for other arguments; the check
//! keeps the text rewrite's output.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: usability
//@codes: CGP-E002

use core::marker::PhantomData;

use cgp::extra::handler::{Computer, ComputerComponent};
use cgp::prelude::*;

pub struct Person;
pub struct Robot;

pub struct Describe;

#[cgp_provider]
impl<Code> Computer<Person, Code, u8> for Describe {
    type Output = u8;
    fn compute(_context: &Person, _code: PhantomData<Code>, input: u8) -> u8 {
        input
    }
}

delegate_components! {
    Robot {
        ComputerComponent: Describe,
    }
}

check_components! {
    Robot {
        ComputerComponent: ((), u16),
    }
}

fn main() {}
