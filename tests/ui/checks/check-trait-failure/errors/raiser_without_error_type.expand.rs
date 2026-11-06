#![feature(prelude_import)]
//! An error raiser wired on a context with no error type, checked for `io::Error`.
//!
//! The headline reads `CanRaiseError<std::io::Error>` over the `[CGP-E113]` cause on
//! `ErrorTypeProviderComponent`: the tool keeps the qualifier rustc prints for the ambiguous
//! `Error`, so the source error does not read as the abstract `Error`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::io;
use cgp::core::error::ErrorRaiserComponent;
use cgp::prelude::*;
pub struct App;
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
            ::core::fmt::Formatter::debug_struct_field1_finish(f, "Error", &&self.0)
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
