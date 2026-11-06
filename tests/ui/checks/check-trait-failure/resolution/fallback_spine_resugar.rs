//! A diagnostic the resolver *declines* still gets its type-level spines resugared by
//! the fallback post-processing chain.
//!
//! `require::<Product![u64, String]>()` asks for `Cons<u64, Cons<String, Nil>>: Needs`, which is not
//! satisfied. No CGP wiring stands behind it (no `check_components!` entry, no provider impl, no
//! context to recover), so every resolver anchor declines and the diagnostic falls through to the
//! text post-processing rather than the typed tree. That chain resugars the raw `Cons`/`Nil` spine
//! back to `Product![u64, String]` (and would fold an all-`Field` spine to the `Struct!`/`Enum!`
//! shape, and an `Either`/`Void` spine to `Sum![…]`), so a declined message reads the same as one
//! the resolver reshaped: the string-level counterpart of the driver's typed `render_ty`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: none

use cgp::prelude::*;

pub trait Needs {}

fn require<T: Needs>() {}

fn main() {
    require::<Product![u64, String]>();
}
