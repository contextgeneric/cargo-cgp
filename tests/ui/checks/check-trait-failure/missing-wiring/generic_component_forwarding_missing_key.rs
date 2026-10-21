//! A forwarding entry for a component with a type parameter, into a table that lacks the key.
//!
//! `ShowerComponent -> Bundle` forwards `CanShow<T>`'s key to a table that does not wire it. The
//! entry names no value for `T`, so no consumer obligation stands for it, and the forwarding-entry
//! anchor declines rather than check `CanShow<()>`, which nothing in the program asks for. The
//! compiler's two errors on the entry stay as rustc wrote them, including its suggestion to enable
//! `trivial_bounds`, which does not fix the program.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: usability
//@codes: none

use cgp::prelude::*;

#[cgp_component(Shower)]
pub trait CanShow<T> {
    fn show(&self, value: T);
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
        ShowerComponent -> Bundle,
    }
}

fn main() {}
