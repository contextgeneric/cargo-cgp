#![feature(prelude_import)]
//! An `open` error table missing the route for `std::io::Error`, in a crate with its own `Error`.
//!
//! `App` routes `ParseIntError` through its open `ErrorRaiserComponent` table, and the check asks for
//! `std::io::Error`. The `[CGP-E107]` leaf names the missing path as
//! `@ErrorRaiserComponent.std::io::Error`, a segment `Path!` accepts as written. rustc prints the
//! type qualified because `Error` names several types here, and the tool keeps the qualifier, so the
//! path does not read as the crate's own `backend::Error` or the abstract `Error`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::num::ParseIntError;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::DebugError;
use cgp::prelude::*;
pub mod backend {
    pub struct Error(pub String);
    #[automatically_derived]
    impl ::core::fmt::Debug for Error {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Error", &&self.0)
        }
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
impl __CheckApp<ErrorRaiserComponent, std::io::Error> for App {}
pub fn unused(_error: backend::Error) {}
fn main() {}
