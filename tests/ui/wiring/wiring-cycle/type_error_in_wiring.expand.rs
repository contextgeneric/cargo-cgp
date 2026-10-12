#![feature(prelude_import)]
//! A wiring value naming a type that does not resolve, beside an `open` dispatch that cycles.
//!
//! The unresolved `backend::Error` makes the error type `{type error}`, which the walk does not
//! descend, so no root cause names it. `@ErrorRaiserComponent.String: DebugError` is a real cycle
//! (`DebugError` raises through `CanRaiseError<String>`), so the overflow is `[CGP-E020]`, beside the
//! `E0433` for `backend`. The `[CGP-E020]` help still names `UseContext`.
//!
//! CGP error class: wiring cycle (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/wiring-cycle.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::num::ParseIntError;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::DebugError;
use cgp::prelude::*;
pub struct App;
impl DelegateComponent<ErrorRaiserComponent> for App {
    type Delegate = RedirectLookup<App, Path!(@ErrorRaiserComponent)>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorRaiserComponent, __Context__, __Params__> for App
where
    RedirectLookup<
        App,
        Path!(@ErrorRaiserComponent),
    >: IsProviderFor<ErrorRaiserComponent, __Context__, __Params__>,
{}
impl DelegateComponent<ErrorTypeProviderComponent> for App {
    type Delegate = UseType<backend::Error>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__> for App
where
    UseType<
        backend::Error,
    >: IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__>,
{}
impl<
    __Wildcard__,
> DelegateComponent<
    PathCons<ErrorRaiserComponent, PathCons<ParseIntError, __Wildcard__>>,
> for App {
    type Delegate = DebugError;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<ErrorRaiserComponent, PathCons<ParseIntError, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    DebugError: IsProviderFor<
        PathCons<ErrorRaiserComponent, PathCons<ParseIntError, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<ErrorRaiserComponent, PathCons<String, __Wildcard__>>>
for App {
    type Delegate = DebugError;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<ErrorRaiserComponent, PathCons<String, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    DebugError: IsProviderFor<
        PathCons<ErrorRaiserComponent, PathCons<String, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<ErrorRaiserComponent, ParseIntError> for App {}
fn main() {}
