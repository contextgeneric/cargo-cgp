#![feature(prelude_import)]
//! Acceptable failure: a context joins `namespace DefaultNamespace;` and then wires a
//! bare, unprefixed component key (`ErrorTypeProviderComponent`) that the namespace
//! already *redirects* — `DefaultNamespace` maps that key, under its `@cgp.core.error`
//! prefix, to a `RedirectLookup`. The bare `DelegateComponent<ErrorTypeProviderComponent>`
//! entry overlaps the namespace's blanket forwarding, so coherence rejects the pair with
//! E0119 (plus a downstream note).
//!
//! Because the namespace's value for this key is a `RedirectLookup`, the tool recovers the
//! redirect target by normalizing `<ErrorTypeProviderComponent as DefaultNamespace<App>>
//! ::Delegate` through the trait solver, so this reads as a *redirect collision* (`[CGP-E007]`)
//! rather than a bare overlap: it drops the redundant `IsProviderFor` half (and rustc's
//! downstream note with it), rewrites the `DelegateComponent` half to `[CGP-E007] component
//! `ErrorTypeProviderComponent` on `App` is redirected to
//! `@cgp.core.error.ErrorTypeProviderComponent``, and adds a `help` naming the provider and the
//! redirected key. If the projection normalization in `resolve/conflict.rs` regresses, this
//! reverts to the `[CGP-E005]` overlap form.
//!
//! See docs/errors/wiring/namespace-override-conflict.md and docs/error-code.md (CGP-E007).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::error::ErrorTypeProviderComponent;
use cgp::prelude::*;
pub struct App;
impl<__Key__, __Value__> DelegateComponent<__Key__> for App
where
    __Key__: DefaultNamespace<App, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
impl<
    __Key__,
    __Value__,
    __Context__,
    __Params__,
> IsProviderFor<__Key__, __Context__, __Params__> for App
where
    __Key__: DefaultNamespace<App, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
impl DelegateComponent<ErrorTypeProviderComponent> for App {
    type Delegate = UseType<String>;
}
impl<
    __Context__,
    __Params__,
> IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__> for App
where
    UseType<String>: IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__>,
{}
fn main() {}
