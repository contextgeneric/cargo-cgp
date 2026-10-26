#![feature(prelude_import)]
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
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct Bar;
pub struct __MyNamespaceComponents;
pub trait MyNamespace<__Table__> {
    type Delegate;
}
impl<__Table__, __Wildcard__> MyNamespace<__Table__>
for PathCons<Symbol!("foo"), PathCons<Bar, __Wildcard__>> {
    type Delegate = RedirectLookup<__Table__, PathCons<Symbol!("baz"), __Wildcard__>>;
}
impl<__Table__, T, __Wildcard__> MyNamespace<__Table__>
for PathCons<Symbol!("foo"), PathCons<T, __Wildcard__>> {
    type Delegate = RedirectLookup<__Table__, PathCons<Symbol!("qux"), __Wildcard__>>;
}
fn main() {}
