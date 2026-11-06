#![feature(prelude_import)]
//! Wrapping an error with a borrowed detail through a wrapper that needs `'static` details.
//!
//! A bare `E0477` "the type `&'a str` does not fulfill the required lifetime" on the check entry,
//! with no chain to the wrapper's `'static` bound.
//!
//! CGP error class: unsatisfied ordinary trait bound (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/ordinary-trait-bound.md).
//!
//! Issue: https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cargo-cgp/issues/usability.md
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use std::io;
use cgp::core::error::{
    ErrorRaiserComponent, ErrorTypeProviderComponent, ErrorWrapperComponent,
};
use cgp::prelude::*;
pub trait CanLoad: HasErrorType {
    fn load(&self, path: &str) -> Result<String, <Self as HasErrorType>::Error>;
}
impl<__Context__> CanLoad for __Context__
where
    __Context__: HasErrorType,
    __Context__: Loader<__Context__>,
{
    fn load(&self, path: &str) -> Result<String, <Self as HasErrorType>::Error> {
        __Context__::load(self, path)
    }
}
pub trait Loader<__Context__>: IsProviderFor<LoaderComponent, __Context__, ()>
where
    __Context__: HasErrorType,
{
    fn load(
        __context__: &__Context__,
        path: &str,
    ) -> Result<String, <__Context__ as HasErrorType>::Error>;
}
impl<__Provider__, __Context__> Loader<__Context__> for __Provider__
where
    __Context__: HasErrorType,
    __Provider__: DelegateComponent<LoaderComponent>
        + IsProviderFor<LoaderComponent, __Context__, ()>,
    <__Provider__ as DelegateComponent<LoaderComponent>>::Delegate: Loader<__Context__>,
{
    fn load(
        __context__: &__Context__,
        path: &str,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        <__Provider__ as DelegateComponent<
            LoaderComponent,
        >>::Delegate::load(__context__, path)
    }
}
pub struct LoaderComponent;
impl<__Context__> Loader<__Context__> for UseContext
where
    __Context__: HasErrorType,
    __Context__: CanLoad,
{
    fn load(
        __context__: &__Context__,
        path: &str,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        __Context__::load(__context__, path)
    }
}
impl<__Context__> IsProviderFor<LoaderComponent, __Context__, ()> for UseContext
where
    __Context__: HasErrorType,
    __Context__: CanLoad,
{}
impl<__Context__, __Components__, __Path__> Loader<__Context__>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasErrorType,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<__Path__>>::Delegate: Loader<__Context__>,
{
    fn load(
        __context__: &__Context__,
        path: &str,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        <__Components__ as DelegateComponent<
            __Path__,
        >>::Delegate::load(__context__, path)
    }
}
impl<
    __Context__,
    __Components__,
    __Path__,
> IsProviderFor<LoaderComponent, __Context__, ()>
for RedirectLookup<__Components__, __Path__>
where
    __Context__: HasErrorType,
    __Components__: DelegateComponent<__Path__>,
    <__Components__ as DelegateComponent<
        __Path__,
    >>::Delegate: IsProviderFor<LoaderComponent, __Context__, ()> + Loader<__Context__>,
{}
impl<__Context__> Loader<__Context__> for LoadFile
where
    __Context__: CanRaiseError<io::Error> + for<'a> CanWrapError<&'a str>,
    __Context__: HasErrorType,
{
    fn load(
        __context__: &__Context__,
        path: &str,
    ) -> Result<String, <__Context__ as HasErrorType>::Error> {
        std::fs::read_to_string(path)
            .map_err(__Context__::raise_error)
            .map_err(|e| __Context__::wrap_error(e, path))
    }
}
impl<__Context__> IsProviderFor<LoaderComponent, __Context__, ()> for LoadFile
where
    __Context__: CanRaiseError<io::Error> + for<'a> CanWrapError<&'a str>,
    __Context__: HasErrorType,
{}
pub struct LoadFile;
pub struct App;
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
impl DelegateComponent<ErrorWrapperComponent> for App {
    type Delegate = backend::RaiseBackendError;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<ErrorWrapperComponent, __Context__, __Params__> for App
where
    backend::RaiseBackendError: IsProviderFor<
        ErrorWrapperComponent,
        __Context__,
        __Params__,
    >,
{}
impl DelegateComponent<LoaderComponent> for App {
    type Delegate = LoadFile;
}
impl<
    __Context__,
    __Params__: ?Sized,
> IsProviderFor<LoaderComponent, __Context__, __Params__> for App
where
    LoadFile: IsProviderFor<LoaderComponent, __Context__, __Params__>,
{}
trait __CheckApp<
    __Component__,
    __Params__: ?Sized,
>: CanUseComponent<__Component__, __Params__> {}
impl __CheckApp<LoaderComponent, ()> for App {}
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
