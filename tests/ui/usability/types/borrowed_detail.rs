//! Wrapping an error with a borrowed detail through a wrapper that needs `'static` details.
//!
//! A bare `E0477` "the type `&'a str` does not fulfill the required lifetime" on the check entry,
//! with no chain to the wrapper's `'static` bound.
//!
//! CGP error class: unsatisfied ordinary trait bound (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/ordinary-trait-bound.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use std::io;

use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent, ErrorWrapperComponent};
use cgp::prelude::*;

#[cgp_component(Loader)]
#[use_type(HasErrorType.Error)]
pub trait CanLoad {
    fn load(&self, path: &str) -> Result<String, Error>;
}

#[cgp_impl(new LoadFile)]
#[uses(CanRaiseError<io::Error>, for<'a> CanWrapError<&'a str>)]
#[use_type(HasErrorType.Error)]
impl Loader {
    fn load(&self, path: &str) -> Result<String, Error> {
        std::fs::read_to_string(path)
            .map_err(Self::raise_error)
            .map_err(|e| Self::wrap_error(e, path))
    }
}

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<backend::Error>,
        [ErrorRaiserComponent, ErrorWrapperComponent]: backend::RaiseBackendError,
        LoaderComponent: LoadFile,
    }
}

check_components! {
    App {
        LoaderComponent,
    }
}

mod backend {
    use core::error::Error as StdError;
    use core::fmt::Display;

    use cgp::core::error::{ErrorRaiser, ErrorRaiserComponent, ErrorWrapper, ErrorWrapperComponent};
    use cgp::prelude::*;

    #[derive(Debug)]
    pub struct Error(pub String);

    pub struct RaiseBackendError;

    #[cgp_impl(RaiseBackendError)]
    #[use_type(HasErrorType.{Error = crate::backend::Error})]
    impl<E> ErrorRaiser<E>
    where
        E: StdError + Send + Sync + 'static,
    {
        fn raise_error(e: E) -> Error {
            crate::backend::Error(e.to_string())
        }
    }

    #[cgp_impl(RaiseBackendError)]
    #[use_type(HasErrorType.{Error = crate::backend::Error})]
    impl<Detail> ErrorWrapper<Detail>
    where
        Detail: Display + Send + Sync + 'static,
    {
        fn wrap_error(error: Error, detail: Detail) -> Error {
            crate::backend::Error(format!("{detail}: {}", error.0))
        }
    }
}

fn main() {}
