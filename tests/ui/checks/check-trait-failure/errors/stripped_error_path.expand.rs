#![feature(prelude_import)]
//! An error raiser that pins the abstract error to a concrete type named `Error` from another module.
//!
//! The context wires `UseType<String>`, so `[CGP-E005]` fires: "expected the abstract type `Error`
//! … to be `backend::Error`", with a help naming `UseType<backend::Error>`. rustc prints the type
//! qualified because `Error` names several types here, and the tool keeps that qualifier, since a
//! bare `Error` would read as the abstract type itself.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::io;
use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::prelude::*;
pub struct App;
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
impl DelegateComponent<ErrorRaiserComponent> for App {
    type Delegate = backend::RaiseBackendError;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorRaiserComponent, __Context__, __Params__> for App
where
    backend::RaiseBackendError: IsProviderFor<
        ErrorRaiserComponent,
        __Context__,
        __Params__,
    >,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<ErrorRaiserComponent, io::Error> for App {}
mod backend {
    use core::error::Error as StdError;
    use core::fmt::Display;
    use cgp::core::error::{
        ErrorRaiser, ErrorRaiserComponent, ErrorWrapper, ErrorWrapperComponent,
    };
    use cgp::prelude::*;
    pub struct Error(pub String);
    #[automatically_derived]
    impl ::core::fmt::Debug for Error {
        #[inline]
        fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
            ::core::fmt::Formatter::debug_tuple_field1_finish(f, "Error", &&self.0)
        }
    }
    pub struct RaiseBackendError;
    impl<__Context__, E> ErrorRaiser<__Context__, E> for RaiseBackendError
    where
        E: StdError + Send + Sync + 'static,
        __Context__: HasErrorType<Error = crate::backend::Error>,
    {
        fn raise_error(e: E) -> <__Context__ as HasErrorType>::Error {
            crate::backend::Error(e.to_string())
        }
    }
    impl<__Context__, E> IsProviderFor<ErrorRaiserComponent, __Context__, (E)>
    for RaiseBackendError
    where
        E: StdError + Send + Sync + 'static,
        __Context__: HasErrorType<Error = crate::backend::Error>,
    {}
    impl<__Context__, Detail> ErrorWrapper<__Context__, Detail> for RaiseBackendError
    where
        Detail: Display + Send + Sync + 'static,
        __Context__: HasErrorType<Error = crate::backend::Error>,
    {
        fn wrap_error(
            error: <__Context__ as HasErrorType>::Error,
            detail: Detail,
        ) -> <__Context__ as HasErrorType>::Error {
            crate::backend::Error(
                ::alloc::__export::must_use({
                    ::alloc::fmt::format(format_args!("{1}: {0}", error.0, detail))
                }),
            )
        }
    }
    impl<__Context__, Detail> IsProviderFor<ErrorWrapperComponent, __Context__, (Detail)>
    for RaiseBackendError
    where
        Detail: Display + Send + Sync + 'static,
        __Context__: HasErrorType<Error = crate::backend::Error>,
    {}
}
fn main() {}
