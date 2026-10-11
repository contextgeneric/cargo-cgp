//! A namespace that inherits itself, `new SelfNs: SelfNs`.
//!
//! `E0207` on `__Value__` across the whole block; decidable at macro time.
//!
//! CGP error class: namespace inheritance cycle, its self-inheriting form (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/namespace-inheritance-cycle.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: none

use cgp::prelude::*;

cgp_namespace! {
    new SelfNs: SelfNs {}
}

fn main() {}
