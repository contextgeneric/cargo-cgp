//! A plain Rust error beside a forwarding entry whose table lacks the key.
//!
//! `GreeterComponent -> Bundle` forwards to a table that does not wire `GreeterComponent`, and
//! `main` passes an `Rc` to a `Send` bound, a separate mistake. The tool reports both: the
//! `[CGP-E001]` block at the entry, and the `Send` error as rustc wrote it. The forwarding-entry
//! anchor reads only an error whose caret holds the entry's forwarding bound.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E114

use std::rc::Rc;

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

fn needs_send<T: Send>(_value: T) {}

fn main() {
    needs_send(Rc::new(1));
}
