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
//! cargo-cgp does not yet reshape the orphan class: it passes rustc's `E0210`
//! through unchanged, without explaining in CGP terms that the mistake is
//! registering a *prefixed* foreign component's default into a foreign namespace.
//! That CGP-level re-presentation is the usability gap this fixture tracks.
//! Bare-marker sibling: `default_impl_foreign_component.rs`.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp/blob/main/docs/errors/wiring/orphan-rule.md>.

//@aux-build: cgp-test-crate-a

use cgp::prelude::*;
use cgp_test_crate_a::{Announcer, AnnouncerComponent, AppNamespace, HasName};

#[cgp_impl(new AnnounceQuietly)]
#[default_impl(@app.AnnouncerComponent in AppNamespace)]
impl Announcer
where
    Self: HasName,
{
    fn announce(&self) -> String {
        format!("(psst, {})", self.name())
    }
}

fn main() {}
