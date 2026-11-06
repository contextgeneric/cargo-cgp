//! `#[derive(CgpVariant)]` on an enum with a variant that has no fields, in a
//! `no_std` crate without `Box` in scope. The mutable extractor builds an empty
//! variant's `&mut Nil` payload with `Box::leak(Box::new(Nil))`, writing `Box`
//! bare so it resolves where the derive is used, because `cgp` links no `alloc`
//! and so cannot name `Box` itself. `extern crate std;` keeps the binary
//! linkable while `#![no_std]` drops the `std` prelude, so `Box` is not in
//! scope and the expansion fails to resolve it, at the empty variant `Closed`.
//! rustc names the missing `Box` and suggests the import, which is the fix: see
//! `ok/no_std_empty_variant.rs`.
//!
//! CGP error class: out-of-scope generated name (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/lowering/out-of-scope-generated-name.md).

//@quality: acceptable
//@codes: none

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
