//! Two namespaces that inherit from each other, which both compilers on the pinned nightly accept.
//!
//! Stable `rustc` rejects the cycle with an `E0275` overflow at each `cgp_namespace!` definition.
//! On the pinned nightly both the default solver and the next-generation solver the tool uses accept
//! it, so plain `cargo check` and `cargo cgp check` both compile it clean. That the tool reports
//! nothing here is a documented limitation: it checks only what the next solver rejects. The
//! fixture's exit-status check fails if a later nightly makes the two compilers disagree.
//!
//! CGP error class: namespace inheritance cycle (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/namespace-inheritance-cycle.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

cgp_namespace! {
    new NamespaceA: NamespaceB {}
}

cgp_namespace! {
    new NamespaceB: NamespaceA {}
}

fn main() {}
