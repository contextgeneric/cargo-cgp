#![feature(prelude_import)]
//! Orphan-rule violation: a `cgp_namespace!` block *without* `new` re-opens a
//! foreign namespace to add an entry keyed on a foreign component. Both
//! `AppNamespace` and `GreeterComponent` come from the auxiliary upstream crate, so
//! the `GreeterComponent => @foo` entry expands to `impl<__Table__>
//! AppNamespace<__Table__> for GreeterComponent { .. }`, whose trait and self type
//! are both foreign and whose `__Table__` parameter no local type covers — the
//! orphan rule rejects it (`E0210`). To extend a foreign namespace, define a *new*
//! local namespace that *inherits* it (`new Local: AppNamespace { .. }`), which is
//! orphan-safe because the emitted impls are for the local trait.
//!
//! cargo-cgp reshapes the orphan class into a `[CGP-E011]` header naming the
//! foreign namespace (`AppNamespace`) and the foreign key (`GreeterComponent`)
//! instead of the machinery parameter `__Table__`. Because the trigger is a
//! `cgp_namespace!` re-open rather than a `#[default_impl]` registration — told
//! apart by the impl's `__Table__` parameter — the `help` gives the re-open fix:
//! define a new local namespace that *inherits* the foreign one. This is the
//! `cgp_namespace!` trigger of the orphan class, alongside the `#[default_impl]`
//! triggers in the sibling fixtures.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp/blob/main/docs/errors/wiring/orphan-rule.md>.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
use cgp_test_crate_a::{AppNamespace, GreeterComponent};
impl<__Table__> AppNamespace<__Table__> for GreeterComponent {
    type Delegate = RedirectLookup<__Table__, Path!(@foo)>;
}
fn main() {}
