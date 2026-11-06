//! A namespace that inherits itself through a `self::` path, `new SelfNs: self::SelfNs`, which
//! `cgp_namespace!` rejects at expansion like the bare form.
//!
//! `self::SelfNs` names the namespace being declared, so the cycle is visible in the one
//! invocation, and the macro reports it with the caret on the parent. The tool must print the
//! macro's message as plain `cargo check` prints it.
//!
//! CGP error class: none; a macro's own rejection, documented with the macro in
//! cgp-knowledge-base/cgp/implementation/entrypoints/cgp_namespace.md.

use cgp::prelude::*;

cgp_namespace! {
    new SelfNs: self::SelfNs {}
}

fn main() {}
