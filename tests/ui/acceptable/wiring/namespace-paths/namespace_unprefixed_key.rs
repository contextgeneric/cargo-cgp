//! Acceptable failure: a context joins `namespace DefaultNamespace;` and then wires a
//! bare, unprefixed component key (`ErrorTypeProviderComponent`) that the namespace
//! already routes under its `@cgp.core.error` prefix. The bare
//! `DelegateComponent<ErrorTypeProviderComponent>` entry overlaps the namespace's
//! blanket forwarding, so coherence rejects the pair with E0119 (plus a downstream
//! note) — the component-marker counterpart of the path override in
//! [override_registered_path.rs].
//!
//! The tool drops the redundant `IsProviderFor` half (and rustc's downstream note with
//! it) and rewrites the `DelegateComponent` half to `[CGP-E004] `App` cannot wire
//! component `ErrorTypeProviderComponent` that is already set through
//! `DefaultNamespace``.
//!
//! See docs/errors/wiring/namespace-override-conflict.md and docs/error-code.md (CGP-E004).

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
