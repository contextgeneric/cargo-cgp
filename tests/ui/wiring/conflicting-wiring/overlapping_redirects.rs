//! Two `delegate_components!` entries that redirect different but overlapping `@`-paths: the
//! generic segment of `@app.<T> T` also matches the `Bar` of `@app.Bar`. The two
//! `DelegateComponent<PathCons<..>>` impls conflict with the coherence error E0119 over the one key
//! they share.
//!
//! Both entries redirect, so the tool takes the pair for one key redirected twice and reports `App`
//! redirecting `@app.T` twice, which is false: each path is redirected once.
//!
//! The redirect counterpart of
//! [path_generic_type_segment_overlap.rs](path_generic_type_segment_overlap.rs), whose two entries
//! name providers.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md.

//@quality: usability
//@codes: CGP-E014

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
