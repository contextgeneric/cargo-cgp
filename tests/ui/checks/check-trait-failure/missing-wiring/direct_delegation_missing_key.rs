//! A `->` delegation to a table that lacks the key.
//!
//! `GreeterComponent -> Bundle` where `Bundle` has no `GreeterComponent` entry. The compiler first
//! reports the entry's own forwarding bound, `Bundle: DelegateComponent<GreeterComponent>`, and
//! the unsized `Delegate` projection it leaves, at the `-> Bundle` the bound is spanned onto. The
//! forwarding-entry anchor seeds `Person: CanGreet`, the consumer the entry wires, so the first of
//! them reads ``[CGP-E001] missing entry for `GreeterComponent` in provider `Bundle` `` over the
//! check's chain, and the second and the check's own report drop as the same failure.
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

check_components! {
    Person {
        GreeterComponent,
    }
}

fn main() {}
