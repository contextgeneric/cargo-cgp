//! An error raiser wired on a context with no error type, checked for `io::Error`.
//!
//! The `[CGP-E107]` cause is right, but the headline reads `CanRaiseError<Error>` for `io::Error`:
//! the stripped path collides with the abstract `Error`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E107

use std::io;

use cgp::core::error::ErrorRaiserComponent;
use cgp::prelude::*;

pub struct App;

delegate_components! {
    App {
        ErrorRaiserComponent: backend::RaiseBackendError,
    }
}

check_components! {
    App {
        ErrorRaiserComponent: io::Error,
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
