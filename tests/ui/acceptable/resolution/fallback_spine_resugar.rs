//! Acceptable: a diagnostic the resolver *declines* still gets its type-level spines resugared by
//! the fallback post-processing chain.
//!
//! `require::<Product![u64, String]>()` asks for `Cons<u64, Cons<String, Nil>>: Needs`, which is not
//! satisfied. There is no CGP wiring behind it — no `check_components!` entry, no provider impl, no
//! context to recover — so all four resolver anchors decline and the diagnostic falls through to the
//! text post-processing rather than the typed tree. That fallback chain now resugars the raw
//! `Cons`/`Nil` spine back to `Product![u64, String]` (and would fold an all-`Field` spine to
//! `Struct! { … }` / `Enum! { … }`, and an `Either`/`Void` spine to `Sum![…]`), so a declined message
//! reads the same as one the resolver reshaped — the string-level counterpart of the driver's typed
//! `render_ty`, catching spines the resolver never traced.

use cgp::prelude::*;

pub trait Needs {}

fn require<T: Needs>() {}

fn main() {
    require::<Product![u64, String]>();
}
