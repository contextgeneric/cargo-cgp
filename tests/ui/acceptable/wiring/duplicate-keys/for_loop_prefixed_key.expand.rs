#![feature(prelude_import)]
//! Acceptable failure: a `for` loop that wires each `ErrorHandlers` entry under the
//! prefixed path `@cgp.core.error.ErrorRaiserComponent.Key`, beside a
//! `namespace AppDefaults;` join that already registers
//! `@cgp.core.error.ErrorRaiserComponent.String`. The loop lowers to a
//! `DelegateComponent<PathCons<.., Key>>` whose generic-tailed key
//! `@cgp.core.error.ErrorRaiserComponent.*` overlaps the namespace forwarding, so
//! coherence rejects the pair with E0119.
//!
//! This is the *prefixed* `for`-key counterpart of the bare-key
//! [for_loop_bare_key.rs]: a bare loop key overlaps *every* key, while a prefixed
//! loop key overlaps only where the prefix path is itself routed by the namespace —
//! here `AppDefaults` registers `@cgp.core.error.ErrorRaiserComponent.String`, so the
//! generic loop tail collides with it. (A prefix the namespace does not register does
//! *not* overlap: the orphan rule proves it, which is why prefixing an otherwise-bare
//! `for` key is the fix.)
//!
//! The tool drops the redundant `IsProviderFor` half and rewrites the
//! `DelegateComponent` half to `[CGP-E005] `App` cannot wire
//! `@cgp.core.error.ErrorRaiserComponent.*` that is already set through
//! `AppDefaults``, exercising the typed path renderer's collapse of a `for`-loop key
//! parameter to a trailing `.*` wildcard.
//!
//! See cgp-knowledge-base/cgp/errors/wiring/namespace-forwarding-conflict.md and
//! cgp-knowledge-base/cargo-cgp/error-code.md (CGP-E005).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::error::ErrorRaiserComponent;
use cgp::extra::error::DisplayError;
use cgp::prelude::*;
pub struct __ErrorHandlersComponents;
pub trait ErrorHandlers<__Table__> {
    type Delegate;
}
impl<__Table__> ErrorHandlers<__Table__> for String {
    type Delegate = DisplayError;
}
pub struct __AppDefaultsComponents;
pub trait AppDefaults<__Table__> {
    type Delegate;
}
impl<__Table__, __Key__, __Value__> AppDefaults<__Table__> for __Key__
where
    __Key__: DefaultNamespace<__AppDefaultsComponents>,
    __Key__: DefaultNamespace<__Table__, Delegate = __Value__>,
{
    type Delegate = __Value__;
}
impl<__Table__, __Wildcard__> AppDefaults<__Table__>
for PathCons<
    Symbol!("cgp"),
    PathCons<
        Symbol!("core"),
        PathCons<
            Symbol!("error"),
            PathCons<ErrorRaiserComponent, PathCons<String, __Wildcard__>>,
        >,
    >,
> {
    type Delegate = DisplayError;
}
pub struct App;
impl<__Key__, __Value__> DelegateComponent<__Key__> for App
where
    __Key__: AppDefaults<App, Delegate = __Value__>,
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
    __Key__: AppDefaults<App, Delegate = __Value__>,
    __Value__: IsProviderFor<__Key__, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
    Key,
    Value,
> DelegateComponent<
    PathCons<
        Symbol!("cgp"),
        PathCons<
            Symbol!("core"),
            PathCons<
                Symbol!("error"),
                PathCons<ErrorRaiserComponent, PathCons<Key, __Wildcard__>>,
            >,
        >,
    >,
> for App
where
    Key: ErrorHandlers<App, Delegate = Value>,
{
    type Delegate = Value;
}
impl<
    __Wildcard__,
    Key,
    Value,
    __Context__,
    __Params__,
> IsProviderFor<
    PathCons<
        Symbol!("cgp"),
        PathCons<
            Symbol!("core"),
            PathCons<
                Symbol!("error"),
                PathCons<ErrorRaiserComponent, PathCons<Key, __Wildcard__>>,
            >,
        >,
    >,
    __Context__,
    __Params__,
> for App
where
    Key: ErrorHandlers<App, Delegate = Value>,
    Value: IsProviderFor<
        PathCons<
            Symbol!("cgp"),
            PathCons<
                Symbol!("core"),
                PathCons<
                    Symbol!("error"),
                    PathCons<ErrorRaiserComponent, PathCons<Key, __Wildcard__>>,
                >,
            >,
        >,
        __Context__,
        __Params__,
    >,
{}
fn main() {}
