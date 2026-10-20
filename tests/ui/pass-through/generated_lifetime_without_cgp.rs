//! A crate without CGP whose own attribute macro adds a reserved lifetime to a struct.
//!
//! `#[generated_lifetime]` (from an auxiliary crate that knows nothing of CGP) gives `Labels` a
//! lifetime parameter its fields do not use, so the field's elided `&str` makes rustc suggest that
//! generated lifetime, a name this file never writes. In a crate that uses CGP the tool drops
//! such a suggestion, since it names a lifetime the reader cannot write. This crate does not use
//! CGP, so the tool must leave the error, suggestion included, exactly as rustc prints it.
//!
//! CGP error class: none; plain Rust the tool passes through.

//@aux-build: cgp-test-plain-macros

use cgp_test_plain_macros::generated_lifetime;

#[generated_lifetime]
pub struct Labels {
    pub first: &str,
}

fn main() {}
