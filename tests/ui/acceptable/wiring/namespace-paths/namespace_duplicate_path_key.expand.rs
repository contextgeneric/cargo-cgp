#![feature(prelude_import)]
//! Acceptable failure: two `@`-path prefix-rewrite entries in one `cgp_namespace!`
//! block that map the same path produce two conflicting `MyNamespace<_>` impls
//! (keyed by the same `PathCons<..>` type), rejected with the coherence error
//! E0119 — the namespace analogue of the `delegate_components!`
//! [duplicate_path_key.rs](../delegate_components/duplicate_path_key.rs).
//!
//! This fixture pins the **error span** for a namespace `@`-path key. The key
//! type is a synthesized `PathCons<..>` nest whose own span points at the macro
//! `call_site`; the entry instead carries the span of the path segments the user
//! wrote, so E0119 lands on the duplicated `@foo.bar` leaf segment rather than on
//! the whole `cgp_namespace!` block. If the re-spanning in
//! `build_namespace_impl` (`mapping/eval.rs`) regresses, the caret snaps back to
//! the block and this `.stderr` changes.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md; error-span
//! mechanics in
//! cgp-knowledge-base/cgp/implementation/entrypoints/cgp_namespace.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
pub struct __MyNamespaceComponents;
pub trait MyNamespace<__Table__> {
    type Delegate;
}
impl<__Table__, __Wildcard__> MyNamespace<__Table__>
for PathCons<Symbol!("foo"), PathCons<Symbol!("bar"), __Wildcard__>> {
    type Delegate = RedirectLookup<__Table__, PathCons<Symbol!("baz"), __Wildcard__>>;
}
impl<__Table__, __Wildcard__> MyNamespace<__Table__>
for PathCons<Symbol!("foo"), PathCons<Symbol!("bar"), __Wildcard__>> {
    type Delegate = RedirectLookup<__Table__, PathCons<Symbol!("qux"), __Wildcard__>>;
}
fn main() {}
