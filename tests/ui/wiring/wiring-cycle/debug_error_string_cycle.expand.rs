#![feature(prelude_import)]
//! `DebugError` wired for the `String` key of an `open` error raiser, so it serves itself.
//!
//! `[CGP-E020]` says the lookup leads back to itself, and its help names `DebugError` as the
//! provider that requires `CanRaiseError<String>` again.
//!
//! CGP error class: wiring cycle (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/wiring-cycle.md).
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
    type Delegate = UseType<String>;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__> for App
where
    UseType<String>: IsProviderFor<ErrorTypeProviderComponent, __Context__, __Params__>,
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
