//! An ordinary impl whose type parameter nothing binds, `impl<T> Describe for Person`.
//!
//! rustc reports E0207, and the tool leaves it as rustc wrote it: the structural detectors claim
//! only an impl CGP's macros generate for a wiring entry or a registration.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self);
}

pub trait Describe {
    fn describe(&self) -> String;
}

pub struct Person;

impl<T> Describe for Person {
    fn describe(&self) -> String {
        String::new()
    }
}

fn main() {}
