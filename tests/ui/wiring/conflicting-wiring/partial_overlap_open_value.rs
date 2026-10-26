//! Two `@`-path keys that overlap in part, where the key they share keeps a type the keys leave
//! open: `@app.<X> Wrap<X>` and `@app.<T> T.Bar` both map `@app.Wrap<X>.Bar` for every `X`, so the
//! two `DelegateComponent<PathCons<..>>` impls conflict with the coherence error E0119.
//!
//! Neither key covers the other, so the tool names the key they share, its open type written `_`:
//! `[CGP-E011] conflicting wiring: `App` wires `@app.Wrap<_>.Bar`, which `@app.Wrap<X>` already
//! covers`, as rustc writes that type in its own message.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E011).

//@quality: acceptable
//@codes: CGP-E011

use cgp::prelude::*;

pub struct Wrap<T>(pub T);

pub struct Bar;

pub struct App;

pub struct First;

pub struct Second;

delegate_components! {
    App {
        @app.<X> Wrap<X>: First,
        @app.<T> T.Bar: Second,
    }
}

fn main() {}
