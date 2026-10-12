//! An error raiser that pins the abstract error to a concrete type named `Error` from another module.
//!
//! The context wires `UseType<String>`, so `[CGP-E005]` fires: "expected the abstract type `Error`
//! … to be `backend::Error`", with a help naming `UseType<backend::Error>`. rustc prints the type
//! qualified because `Error` names several types here, and the tool keeps that qualifier, since a
//! bare `Error` would read as the abstract type itself.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E005 CGP-E101 CGP-E102 CGP-E116

use std::io;

use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::prelude::*;

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
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
