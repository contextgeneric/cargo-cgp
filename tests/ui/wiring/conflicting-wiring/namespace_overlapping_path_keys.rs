//! Two `@`-path entries in one `cgp_namespace!` block whose paths differ but overlap: the generic
//! segment of `@foo.<T> T` also matches the `Bar` of `@foo.Bar`. The two `MyNamespace<_>` impls
//! conflict with the coherence error E0119 over the one key they share.
//!
//! Both entries redirect, so the tool takes the pair for one key redirected twice and reports
//! `MyNamespace` redirecting `@foo.T` twice, which is false: each path is redirected once.
//!
//! The namespace counterpart of [overlapping_redirects.rs](overlapping_redirects.rs).
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md.

//@quality: usability
//@codes: CGP-E014

use cgp::prelude::*;

pub struct Bar;

cgp_namespace! {
    new MyNamespace {
        @foo.Bar => @baz,
        @foo.<T> T => @qux,
    }
}

fn main() {}
