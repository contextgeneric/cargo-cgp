//! Two `@`-path entries in one `cgp_namespace!` block whose paths differ but overlap: the generic
//! segment of `@foo.<T> T` also matches the `Bar` of `@foo.Bar`. The two `MyNamespace<_>` impls
//! conflict with the coherence error E0119 over the one key they share.
//!
//! Both entries redirect, but each redirects its own path once, so the tool words the pair as an
//! overlap, `[CGP-E011] conflicting wiring: `MyNamespace` wires `@foo.Bar`, which `@foo.T` already
//! covers`, naming the generic path as the one that covers the other.
//!
//! The namespace counterpart of [overlapping_redirects.rs](overlapping_redirects.rs).
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E011).

//@quality: acceptable
//@codes: CGP-E011

use cgp::prelude::*;

pub struct Bar;

cgp_namespace! {
    new MyNamespace {
        @foo.Bar => @baz,
        @foo.<T> T => @qux,
    }
}

fn main() {}
