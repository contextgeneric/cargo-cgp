#![feature(prelude_import)]
//! Orphan-rule violation: a crate cannot register a default for a *prefixed*
//! upstream component into the upstream namespace with `#[default_impl]`. The
//! auxiliary crate's `Announcer` carries `#[prefix(@app in DefaultNamespace)]`, so
//! its namespace key is the path `@app.AnnouncerComponent`. Registering a default
//! at that path expands to `impl AppNamespace<_> for PathCons<Symbol<"app">,
//! PathCons<AnnouncerComponent, Nil>>`, whose trait and every element of the `Self`
//! type are foreign to this crate — an orphan-rule violation (`E0210`). A default
//! keyed on a *prefix path* can therefore only be written in the crate that owns
//! the namespace.
//!
//! cargo-cgp reshapes the orphan class into a `[CGP-E011]` header naming the
//! foreign namespace (`AppNamespace`) and the foreign *path* key
//! (`@app.AnnouncerComponent`, resugared from the `PathCons<Symbol<…>>` spine)
//! instead of the machinery parameter `__Components__`, with the ownership-based
//! fix in a `help`. The path key is what distinguishes this from its bare-marker
//! sibling `default_impl_foreign_component.rs`.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp/blob/main/docs/errors/wiring/orphan-rule.md>.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
use cgp_test_crate_a::{Announcer, AnnouncerComponent, AppNamespace, HasName};
impl<__Context__> Announcer<__Context__> for AnnounceQuietly
where
    __Context__: HasName,
{
    fn announce(__context__: &__Context__) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("(psst, {0})", __context__.name()))
        })
    }
}
impl<__Context__> IsProviderFor<AnnouncerComponent, __Context__, ()> for AnnounceQuietly
where
    __Context__: HasName,
{}
pub struct AnnounceQuietly;
impl<__Components__> AppNamespace<__Components__> for Path!(@app.AnnouncerComponent) {
    type Delegate = AnnounceQuietly;
}
fn main() {}
