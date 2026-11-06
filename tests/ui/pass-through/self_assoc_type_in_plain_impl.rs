//! A hand-written impl naming `Self::Error`, which its trait does not declare.
//!
//! rustc reports E0220, and the tool leaves it as rustc wrote it: `[CGP-E035]` claims only a
//! definition a CGP macro generated.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.

use cgp::prelude::*;

#[cgp_component(Greeter)]
pub trait CanGreet {
    fn greet(&self) -> String;
}

pub trait Describe {
    fn describe(&self) -> String;
}

pub struct Person;

impl Describe for Person {
    fn describe(&self) -> String {
        let _error: Option<Self::Error> = None;
        String::new()
    }
}

fn main() {}
