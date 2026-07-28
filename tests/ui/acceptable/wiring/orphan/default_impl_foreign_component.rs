//! Orphan-rule violation: a crate registers a `#[default_impl]` for a foreign
//! *unprefixed* component into a foreign namespace. Both `GreeterComponent` and
//! `AppNamespace` come from the auxiliary upstream crate, so the generated
//! `impl<__Components__> AppNamespace<__Components__> for GreeterComponent` has a
//! foreign trait and a foreign self type, with no local type covering
//! `__Components__` — the orphan rule rejects it (`E0210`). Registering a
//! per-component default needs the crate to own *either* the namespace or the
//! component key; owning neither, this crate cannot.
//!
//! cargo-cgp reshapes the orphan class into a `[CGP-E011]` header naming the
//! foreign namespace (`AppNamespace`) and the foreign key (`GreeterComponent`)
//! instead of the machinery parameter `__Components__`, and carries the
//! ownership-based fix in a `help` — own one end of the wiring, by keying it on a
//! local component or registering it from the namespace's own crate. This is the
//! bare-marker sibling of `default_impl_foreign_prefix_path.rs`; the orphan-*safe*
//! counterpart is the positive `ok/cross_crate_wiring.rs` fixture.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/orphan-rule.md>.

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
