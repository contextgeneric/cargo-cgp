//! `#[derive(CgpVariant)]` on an enum with a variant that has no fields, in a
//! `no_std` crate that brings `Box` into scope. The mutable extractor builds an
//! empty variant's `&mut Nil` payload with `Box::leak(Box::new(Nil))`, writing
//! `Box` bare so it resolves where the derive is used, and here it resolves to
//! the imported `std::boxed::Box`. `extern crate std;` keeps the binary linkable
//! while `#![no_std]` drops the `std` prelude, so the import is what puts `Box`
//! in scope. The program compiles clean.

#![no_std]

extern crate std;

use std::boxed::Box;

use cgp::prelude::*;

#[derive(CgpVariant)]
pub enum Status {
    Active(u64),
    Closed,
}

fn main() {
    let mut status = Status::Closed;
    let _ = status.extractor_mut();
    let _ = Status::Active(1);
}
