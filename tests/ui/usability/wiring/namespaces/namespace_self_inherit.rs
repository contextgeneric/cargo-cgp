//! A namespace that inherits itself, `new SelfNs: SelfNs`.
//!
//! `E0207` on `__Value__` across the whole block; decidable at macro time.
//!
//! CGP error class: unconstrained generic (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/unconstrained-generic.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use cgp::prelude::*;

cgp_namespace! {
    new SelfNs: SelfNs {}
}

fn main() {}
