#![feature(prelude_import)]
//! Acceptable failure: two `@`-path entries where one is a prefix of the other
//! (`@cgp.core.error.ErrorTypeProviderComponent` and
//! `@cgp.core.error.ErrorTypeProviderComponent.String`). Each path entry lowers to a
//! `DelegateComponent<PathCons<.., __Wildcard__>>` with an open wildcard tail, so the
//! shorter path's wildcard covers the longer one and the two overlap with the
//! coherence error E0119 — the sub-key shape of the duplicate-path conflict
//! ([delegate_duplicate_path_key.rs] maps the *same* path twice).
//!
//! The tool drops the redundant `IsProviderFor` half and rewrites the
//! `DelegateComponent` half to `[CGP-E005] `App` cannot wire
//! `@cgp.core.error.ErrorTypeProviderComponent.String.*` that is already set
//! through `@cgp.core.error.ErrorTypeProviderComponent.*``, naming both
//! overlapping paths in bare `@…` form (the wildcard tail is what makes the
//! overlap visible).
//!
//! See cgp-knowledge-base/cgp/errors/wiring/conflicting-wiring.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E005).
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
impl<
    __Wildcard__,
> DelegateComponent<
    PathCons<
        Symbol!("cgp"),
        PathCons<
            Symbol!("core"),
            PathCons<
                Symbol!("error"),
                PathCons<ErrorTypeProviderComponent, __Wildcard__>,
            >,
        >,
    >,
> for App {
    type Delegate = UseType<String>;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<
        Symbol!("cgp"),
        PathCons<
            Symbol!("core"),
            PathCons<
                Symbol!("error"),
                PathCons<ErrorTypeProviderComponent, __Wildcard__>,
            >,
        >,
    >,
    __Context__,
    __Params__,
> for App
where
    UseType<
        String,
    >: IsProviderFor<
        PathCons<
            Symbol!("cgp"),
            PathCons<
                Symbol!("core"),
                PathCons<
                    Symbol!("error"),
                    PathCons<ErrorTypeProviderComponent, __Wildcard__>,
                >,
            >,
        >,
        __Context__,
        __Params__,
    >,
{}
impl<
    __Wildcard__,
> DelegateComponent<
    PathCons<
        Symbol!("cgp"),
        PathCons<
            Symbol!("core"),
            PathCons<
                Symbol!("error"),
                PathCons<ErrorTypeProviderComponent, PathCons<String, __Wildcard__>>,
            >,
        >,
    >,
> for App {
    type Delegate = UseType<String>;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<
        Symbol!("cgp"),
        PathCons<
            Symbol!("core"),
            PathCons<
                Symbol!("error"),
                PathCons<ErrorTypeProviderComponent, PathCons<String, __Wildcard__>>,
            >,
        >,
    >,
    __Context__,
    __Params__,
> for App
where
    UseType<
        String,
    >: IsProviderFor<
        PathCons<
            Symbol!("cgp"),
            PathCons<
                Symbol!("core"),
                PathCons<
                    Symbol!("error"),
                    PathCons<ErrorTypeProviderComponent, PathCons<String, __Wildcard__>>,
                >,
            >,
        >,
        __Context__,
        __Params__,
    >,
{}
fn main() {}
