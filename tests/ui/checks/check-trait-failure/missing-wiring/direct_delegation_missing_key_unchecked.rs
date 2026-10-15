//! A `->` delegation to a table that lacks the key, with no check or call that uses it.
//!
//! The compiler reports the entry's own forwarding bound,
//! `Bundle: DelegateComponent<GreeterComponent>`, and the unsized `Delegate` projection it leaves,
//! at the `-> Bundle` the bound is spanned onto; with nothing else to report the mistake, these are
//! the only errors. The forwarding-entry anchor seeds `Person: CanGreet`, the consumer the entry
//! wires, so the first reads
//! ``[CGP-E001] missing entry for `GreeterComponent` in provider `Bundle` ``, and the second drops
//! as the same failure.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E114

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

fn main() {}
