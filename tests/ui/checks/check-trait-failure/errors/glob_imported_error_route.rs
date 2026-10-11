//! An `open` error table missing the route for `std::io::Error`, in a crate that glob-imports its own
//! `Error`.
//!
//! `App` routes `ParseIntError` through its open `ErrorRaiserComponent` table, and the check asks for
//! `std::io::Error`. The crate glob-imports a prelude that re-exports `backend::Error`, so the bare
//! name `Error` in this crate means `backend::Error`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: usability
//@codes: CGP-E001 CGP-E101 CGP-E104 CGP-E107

use core::num::ParseIntError;

use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::DebugError;
use prelude::*;

pub mod backend {
    #[derive(Debug)]
    pub struct Error(pub String);
}

pub mod prelude {
    pub use cgp::prelude::*;

    pub use crate::backend::Error;
}

pub struct App;

delegate_components! {
    App {
        open ErrorRaiserComponent;

        ErrorTypeProviderComponent: UseType<String>,
        @ErrorRaiserComponent.ParseIntError: DebugError,
        @ErrorRaiserComponent.String: DebugError,
    }
}

check_components! {
    App {
        ErrorRaiserComponent: std::io::Error,
    }
}

pub fn unused(_error: Error) {}

fn main() {}
