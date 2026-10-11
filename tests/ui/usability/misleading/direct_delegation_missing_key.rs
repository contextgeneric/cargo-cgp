//! A `->` delegation to a table that lacks the key.
//!
//! `GreeterComponent -> Bundle` where `Bundle` has no `GreeterComponent` entry gives two raw `E0277`s
//! spanning the whole block with a wrong `trivial_bounds` suggestion, then `[CGP-E107]` "context
//! `Person` does not contain any delegate entry", which is false: `Person` has the entry and
//! `Bundle` lacks it.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

#[cgp_component(Farewell)]
pub trait CanFarewell {
    fn farewell(&self);
}

#[cgp_impl(new SayBye)]
impl Farewell {
    fn farewell(&self) {}
}

delegate_components! {
    new Bundle {
        FarewellComponent: SayBye,
    }
}

pub struct Person;

delegate_components! {
    Person {
        GreeterComponent -> Bundle,
    }
}

check_components! {
    Person {
        GreeterComponent,
    }
}

fn main() {}
