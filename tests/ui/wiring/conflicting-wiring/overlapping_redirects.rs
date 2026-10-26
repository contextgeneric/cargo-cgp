//! Two `delegate_components!` entries that redirect different but overlapping `@`-paths: the
//! generic segment of `@app.<T> T` also matches the `Bar` of `@app.Bar`. The two
//! `DelegateComponent<PathCons<..>>` impls conflict with the coherence error E0119 over the one key
//! they share.
//!
//! Both entries redirect, but each redirects its own path once, so the tool words the pair as an
//! overlap, `[CGP-E011] conflicting wiring: `App` wires `@app.Bar`, which `@app.T` already covers`,
//! rather than as one key redirected twice ([duplicate_redirect.rs](duplicate_redirect.rs)).
//!
//! The redirect counterpart of
//! [path_generic_type_segment_overlap.rs](path_generic_type_segment_overlap.rs), whose two entries
//! name providers.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E011).

//@quality: acceptable
//@codes: CGP-E011

use cgp::prelude::*;

pub struct Bar;

pub struct App;

delegate_components! {
    App {
        @app.Bar => @app.first,
        @app.<T> T => @app.second,
    }
}

fn main() {}
