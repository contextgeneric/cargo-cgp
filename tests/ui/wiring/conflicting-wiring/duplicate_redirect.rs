//! The same component redirected twice with explicit `=>` mappings, to two
//! different paths. Each `FooComponent => @app.foo` lowers to a `DelegateComponent<FooComponent>`
//! whose `Delegate` is a `RedirectLookup`, so the two conflict with the coherence error E0119 — a
//! *duplicate redirect*, distinct from a redirect that collides with a direct wiring
//! ([duplicate_open_key.rs]).
//!
//! The tool recognizes that *both* conflicting entries redirect the same key: it drops the
//! redundant `IsProviderFor` half and rewrites the `DelegateComponent` half to `[CGP-E014]
//! conflicting wiring: `App` redirects component `FooComponent` twice, to `@app.foo` and to
//! `@app.bar``,
//! naming both redirect targets, while keeping rustc's two carets on the `FooComponent =>` lines.
//! If the redirect detection in `resolve/conflict.rs` regresses (counting one redirect instead of
//! two), the header reverts to the single-redirect `[CGP-E013]` form.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E014).

//@quality: acceptable
//@codes: CGP-E014

use cgp::prelude::*;

#[cgp_component(Foo)]
pub trait CanFoo {
    fn foo(&self);
}

pub struct App;

delegate_components! {
    App {
        FooComponent =>
            @app.foo,
        FooComponent =>
            @app.bar,
    }
}

fn main() {}
