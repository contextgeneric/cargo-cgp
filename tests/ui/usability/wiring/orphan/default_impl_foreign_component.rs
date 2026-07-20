//! Orphan-rule violation: a crate registers a `#[default_impl]` for a foreign
//! *unprefixed* component into a foreign namespace. Both `GreeterComponent` and
//! `AppNamespace` come from the auxiliary upstream crate, so the generated
//! `impl<__Components__> AppNamespace<__Components__> for GreeterComponent` has a
//! foreign trait and a foreign self type, with no local type covering
//! `__Components__` — the orphan rule rejects it (`E0210`). Registering a
//! per-component default needs the crate to own *either* the namespace or the
//! component key; owning neither, this crate cannot.
//!
//! cargo-cgp does not yet reshape the orphan class: it passes rustc's `E0210`
//! through unchanged, so the message names the machinery parameter `__Components__`
//! and points at the generated impl without explaining, in CGP terms, that the
//! mistake is registering a default into a *foreign* namespace. That CGP-level
//! re-presentation is the usability gap this fixture tracks. This is the
//! bare-marker sibling of `default_impl_foreign_prefix_path.rs`; the orphan-*safe*
//! counterpart is the positive `ok/cross_crate_wiring.rs` fixture.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp/blob/main/docs/errors/wiring/orphan-rule.md>.

//@aux-build: cgp-test-crate-a

use cgp::prelude::*;
use cgp_test_crate_a::{AppNamespace, Greeter, GreeterComponent, HasName};

#[cgp_impl(new GreetPolitely)]
#[default_impl(GreeterComponent in AppNamespace)]
impl Greeter
where
    Self: HasName,
{
    fn greet(&self) -> String {
        format!("Good day, {}", self.name())
    }
}

fn main() {}
