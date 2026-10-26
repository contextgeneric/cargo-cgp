//! Names that resolve to nothing in impls of ordinary traits, in a crate that uses CGP.
//!
//! `impl Add<u32> for Meters` gives its associated type an undeclared `Output`, and
//! `impl Debug for Absent` implements a trait for an undeclared type. Neither impl is a wiring
//! entry or a `#[cgp_impl]` header, so the tool leaves the `E0425`s as rustc wrote them, though one
//! sits where a wiring value would and the other where a `#[cgp_impl]` provider would.

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

pub struct Meters(pub u32);

impl core::ops::Add<u32> for Meters {
    type Output = Missing;

    fn add(self, other: u32) -> Self::Output {
        todo!("{}", self.0 + other)
    }
}

impl core::fmt::Debug for Absent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("absent")
    }
}

fn main() {}
