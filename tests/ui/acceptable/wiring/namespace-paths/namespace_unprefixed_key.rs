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
//! See cgp-knowledge-base/cgp/errors/wiring/namespace-override-conflict.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E007).

use cgp::core::error::ErrorTypeProviderComponent;
use cgp::prelude::*;

pub struct App;

delegate_components! {
    App {
        namespace DefaultNamespace;

        ErrorTypeProviderComponent: UseType<String>,
    }
}

fn main() {}
