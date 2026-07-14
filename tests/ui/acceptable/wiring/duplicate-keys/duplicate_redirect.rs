//! Acceptable failure: the same component redirected twice with explicit `=>` mappings, to two
//! different paths. Each `FooComponent => @app.foo` lowers to a `DelegateComponent<FooComponent>`
//! whose `Delegate` is a `RedirectLookup`, so the two conflict with the coherence error E0119 — a
//! *duplicate redirect*, distinct from a redirect that collides with a direct wiring
//! ([duplicate_open_key.rs]).
//!
//! The tool recognizes that *both* conflicting entries redirect the same key: it drops the
//! redundant `IsProviderFor` half and rewrites the `DelegateComponent` half to `[CGP-E008]
//! duplicate redirect for component `FooComponent` … redirected to both `@app.foo` and `@app.bar``,
//! naming both redirect targets, while keeping rustc's two carets on the `FooComponent =>` lines.
//! If the redirect detection in `resolve/conflict.rs` regresses (counting one redirect instead of
//! two), the header reverts to the single-redirect `[CGP-E007]` form.
//!
//! See docs/errors/wiring/conflicting-wiring.md and docs/error-code.md (CGP-E008).

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
