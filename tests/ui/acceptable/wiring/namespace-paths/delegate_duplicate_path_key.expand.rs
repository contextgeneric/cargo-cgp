#![feature(prelude_import)]
//! Acceptable failure: two `@`-path entries in one `delegate_components!` block
//! that map the same namespace path produce two conflicting `DelegateComponent`
//! impls (keyed by the same `PathCons<..>` type) for `App`, rejected with the
//! coherence error E0119 — the `@`-path analogue of [duplicate_key_same_block.rs].
//!
//! This fixture pins the **error span** for an `@`-path key. The key type is a
//! synthesized `PathCons<..>` nest whose own span points at the macro
//! `call_site`; the entry instead carries the span of the path segments the user
//! wrote, so E0119 lands on the duplicated `ErrorTypeProviderComponent` segment
//! (the path's leaf) rather than on the whole block. If the path-key span
//! threading in `key/path.rs` regresses, the caret snaps back to the block and
//! this `.stderr` changes.
//!
//! See docs/errors/wiring/conflicting-wiring.md; error-span mechanics in docs/implementation/entrypoints/delegate_components.md.
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
fn main() {}
