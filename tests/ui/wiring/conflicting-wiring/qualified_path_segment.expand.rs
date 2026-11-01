#![feature(prelude_import)]
//! A duplicated `@`-path key whose segment needs its qualifier, in a crate with an `Error` of its
//! own.
//!
//! `std::io::Error` beside a local `Error` is printed qualified, since the bare name is ambiguous,
//! and the message keeps the qualifier, so the key reads as its entry wrote it and names the type
//! it wires. A segment cut to `Error` would name the crate's own type.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md>.
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::{DebugError, RaiseFrom};
use cgp::prelude::*;
pub struct Error;
#[automatically_derived]
impl ::core::fmt::Debug for Error {
    #[inline]
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        ::core::fmt::Formatter::write_str(f, "Error")
    }
}
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
    PathCons<ErrorRaiserComponent, PathCons<std::io::Error, __Wildcard__>>,
> for App {
    type Delegate = DebugError;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<ErrorRaiserComponent, PathCons<std::io::Error, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    DebugError: IsProviderFor<
        PathCons<ErrorRaiserComponent, PathCons<std::io::Error, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    __Wildcard__,
> DelegateComponent<
    PathCons<ErrorRaiserComponent, PathCons<std::io::Error, __Wildcard__>>,
> for App {
    type Delegate = RaiseFrom;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<ErrorRaiserComponent, PathCons<std::io::Error, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    RaiseFrom: IsProviderFor<
        PathCons<ErrorRaiserComponent, PathCons<std::io::Error, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
impl<
    __Wildcard__,
> DelegateComponent<PathCons<ErrorRaiserComponent, PathCons<Error, __Wildcard__>>>
for App {
    type Delegate = DebugError;
}
impl<
    __Wildcard__,
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<
    PathCons<ErrorRaiserComponent, PathCons<Error, __Wildcard__>>,
    __Context__,
    __Params__,
> for App
where
    DebugError: IsProviderFor<
        PathCons<ErrorRaiserComponent, PathCons<Error, __Wildcard__>>,
        __Context__,
        __Params__,
    >,
{}
fn main() {}
