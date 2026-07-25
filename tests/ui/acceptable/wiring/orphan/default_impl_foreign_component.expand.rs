#![feature(prelude_import)]
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
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::prelude::*;
use cgp_test_crate_a::{AppNamespace, Greeter, GreeterComponent, HasName};
impl<__Context__> Greeter<__Context__> for GreetPolitely
where
    __Context__: HasName,
{
    fn greet(__context__: &__Context__) -> String {
        ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("Good day, {0}", __context__.name()))
        })
    }
}
impl<__Context__> IsProviderFor<GreeterComponent, __Context__, ()> for GreetPolitely
where
    __Context__: HasName,
{}
pub struct GreetPolitely;
impl<__Components__> AppNamespace<__Components__> for GreeterComponent {
    type Delegate = GreetPolitely;
}
fn main() {}
