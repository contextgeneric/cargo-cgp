#![feature(prelude_import)]
//! An `open` error table missing the route for a dependency's error, in a crate that glob-imports
//! an error of the same name from a module named after that dependency.
//!
//! `App` routes `ParseIntError` through its open `ErrorRaiserComponent` table, and the check asks for
//! `cgp_test_foreign_pipeline::IoFailure`, an error the dependency defines at its root. The crate
//! glob-imports a prelude that re-exports `backend::cgp_test_foreign_pipeline::IoFailure`, so the
//! bare name `IoFailure` in this crate means that item, whose path ends in the dependency's path.
//! rustc prints the dependency's error from its crate name, so the tool keeps the qualifier: the
//! headline reads `CanRaiseError<cgp_test_foreign_pipeline::IoFailure>` and the leaf
//! `@ErrorRaiserComponent.cgp_test_foreign_pipeline::IoFailure`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use core::num::ParseIntError;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::DebugError;
use prelude::*;
pub mod backend {
    pub mod cgp_test_foreign_pipeline {
        pub struct IoFailure;
        #[automatically_derived]
        impl ::core::fmt::Debug for IoFailure {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(f, "IoFailure")
            }
        }
    }
}
pub mod prelude {
    pub use cgp::prelude::*;
    pub use crate::backend::cgp_test_foreign_pipeline::*;
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
impl __CheckApp<ErrorRaiserComponent, cgp_test_foreign_pipeline::IoFailure> for App {}
pub fn unused(_error: IoFailure) {}
fn main() {}
