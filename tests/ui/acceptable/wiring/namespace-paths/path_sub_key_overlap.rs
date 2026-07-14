//! Acceptable failure: two `@`-path entries where one is a prefix of the other
//! (`@cgp.core.error.ErrorTypeProviderComponent` and
//! `@cgp.core.error.ErrorTypeProviderComponent.String`). Each path entry lowers to a
//! `DelegateComponent<PathCons<.., __Wildcard__>>` with an open wildcard tail, so the
//! shorter path's wildcard covers the longer one and the two overlap with the
//! coherence error E0119 — the sub-key shape of the duplicate-path conflict
//! ([delegate_duplicate_path_key.rs] maps the *same* path twice).
//!
//! The tool drops the redundant `IsProviderFor` half and rewrites the
//! `DelegateComponent` half to `[CGP-E004] `App` cannot wire
//! `Path!(@cgp.core.error.ErrorTypeProviderComponent.String.*)` that is already set
//! through `Path!(@cgp.core.error.ErrorTypeProviderComponent.*)``, naming both
//! overlapping paths in their resugared form (the wildcard tail is what makes the
//! overlap visible).
//!
//! See docs/errors/wiring/conflicting-wiring.md and docs/error-code.md (CGP-E004).

use cgp::core::error::ErrorTypeProviderComponent;
use cgp::prelude::*;

pub struct App;

delegate_components! {
    App {
        namespace DefaultNamespace;

        @cgp.core.error.ErrorTypeProviderComponent: UseType<String>,
        @cgp.core.error.ErrorTypeProviderComponent.String: UseType<String>,
    }
}

fn main() {}
