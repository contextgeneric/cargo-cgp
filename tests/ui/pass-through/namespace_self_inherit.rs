//! A namespace that inherits itself, `new SelfNs: SelfNs`, which `cgp_namespace!` rejects at
//! expansion.
//!
//! The cycle is visible in the one invocation, so the macro reports it with the caret on the parent.
//! The tool must print the macro's message as plain `cargo check` prints it.
//!
//! CGP error class: none; a macro's own rejection, documented with the macro in
//! cgp-knowledge-base/cgp/implementation/entrypoints/cgp_namespace.md.

use cgp::prelude::*;

cgp_namespace! {
    new SelfNs: SelfNs {}
}

fn main() {}
