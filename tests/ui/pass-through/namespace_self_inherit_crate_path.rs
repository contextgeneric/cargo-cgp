//! A namespace that inherits itself through a `crate::` path, `new SelfNs: crate::SelfNs`, which
//! `cgp_namespace!` does not check.
//!
//! The macro compares the parent with the namespace only for a bare name or a `self::` path, since
//! any other path names an item it cannot resolve. The expansion reaches the compiler, which
//! rejects it with `E0207` on the generated `__Value__`. The tool must print that error as plain
//! `cargo check` prints it.
//!
//! CGP error class: none; a macro limit, documented with the macro in
//! cgp-knowledge-base/cgp/implementation/entrypoints/cgp_namespace.md.

use cgp::prelude::*;

cgp_namespace! {
    new SelfNs: crate::SelfNs {}
}

fn main() {}
