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
//! cargo-cgp does not yet reshape the orphan class: it passes rustc's `E0210`
//! through unchanged, so the message names the machinery parameter `__Table__`
//! without explaining in CGP terms that the mistake is re-opening a *foreign*
//! namespace. That CGP-level re-presentation is the usability gap this fixture
//! tracks. This is the `cgp_namespace!` trigger of the orphan class, alongside the
//! `#[default_impl]` triggers in the sibling fixtures.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp/blob/main/docs/errors/wiring/orphan-rule.md>.

//@aux-build: cgp-test-crate-a

use cgp::prelude::*;
use cgp_test_crate_a::{AppNamespace, GreeterComponent};

cgp_namespace! {
    AppNamespace {
        GreeterComponent => @foo,
    }
}

fn main() {}
