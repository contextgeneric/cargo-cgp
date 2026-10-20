#![feature(prelude_import)]
//! A crate without CGP whose own attribute macro adds a reserved lifetime, `'__a__`.
//!
//! `#[generated_lifetime]` (from an auxiliary crate that knows nothing of CGP) gives `Labels` a
//! lifetime parameter its fields do not use, so the field's elided `&str` makes rustc suggest the
//! generated `'__a__`, a name this file never writes. In a crate that uses CGP the tool drops such a
//! suggestion, since it names a lifetime the reader cannot write. This crate does not use CGP, so
//! the tool must leave the error, suggestion included, exactly as rustc prints it.
//!
//! CGP error class: none; plain Rust the tool passes through.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp_test_plain_macros::generated_lifetime;
pub struct Labels<'__a__> {
    pub first: &str,
    __marker: ::core::marker::PhantomData<&'__a__ ()>,
}
fn main() {}
