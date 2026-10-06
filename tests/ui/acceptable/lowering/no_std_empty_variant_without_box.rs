//! `#[derive(CgpVariant)]` on an enum with a variant that has no fields, in a
//! `no_std` crate without `Box` in scope. The mutable extractor builds an empty
//! variant's `&mut Nil` payload with `Box::leak(Box::new(Nil))`, writing `Box`
//! bare so it resolves where the derive is used, because `cgp` links no `alloc`
//! and so cannot name `Box` itself. `extern crate std;` keeps the binary
//! linkable while `#![no_std]` drops the `std` prelude, so `Box` is not in
//! scope and the expansion fails to resolve it, at the empty variant `Closed`.
//! rustc names the missing `Box` and suggests the import, which is the fix: see
//! `ok/no_std_empty_variant.rs`.

#![no_std]

extern crate std;

use cgp::prelude::*;

#[derive(CgpVariant)]
pub enum Status {
    Active(u64),
    Closed,
}

fn main() {
    let _ = Status::Closed;
}
