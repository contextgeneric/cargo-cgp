//! Two namespaces that each inherit another instantiation of themselves: `Ns<T>: Ns<u32>` and
//! `Ns<T>: Ns<Vec<T>>`.
//!
//! A namespace's every instantiation has the same parent rule, so a lookup that reaches the parent
//! asks the same namespace again, at `u32` forever or at a type that grows, and neither namespace
//! can resolve a key its table lacks. Stable `rustc` rejects both with an overflow, one of them
//! without a location; the pinned nightly, which the tool and the baseline run, accepts them, as it
//! accepts a namespace inheritance cycle. The tool must print what plain `cargo check` prints.
//!
//! CGP error class: none; a cycle visible in one invocation, documented with the macro in
//! cgp-knowledge-base/cgp/implementation/entrypoints/cgp_namespace.md.

use cgp::prelude::*;

cgp_namespace! {
    <T> new FixedNs<T>: FixedNs<u32> {}
}

cgp_namespace! {
    <T> new GrowingNs<T>: GrowingNs<Vec<T>> {}
}

fn main() {}
