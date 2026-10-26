//! An ordinary impl with a type parameter nothing binds, `impl<T> Describe for Person`, in a crate
//! that uses CGP. rustc reports E0207, and the tool leaves it as rustc wrote it: the structural
//! detectors claim only an impl CGP's macros generate for a wiring entry or a registration.

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
