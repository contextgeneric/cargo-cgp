//! A `->` delegation to a table that lacks the key.
//!
//! `GreeterComponent -> Bundle` where `Bundle` has no `GreeterComponent` entry. The check's
//! `[CGP-E001]` block ends at `[CGP-E110]` "provider `Bundle` does not contain any delegate entry
//! for `GreeterComponent`": `Person` has the entry, and the walk follows its forwarding bound to the
//! table that lacks the key. Two raw `E0277`s still precede it, spanning the whole block with a wrong
//! `trivial_bounds` suggestion, because the `->` entry's impls are not re-spanned onto the entry.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E001 CGP-E101 CGP-E110

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
