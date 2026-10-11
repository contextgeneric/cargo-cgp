//! An `open` error table missing the route for `std::io::Error`, in a crate with its own `Error`.
//!
//! `App` routes `ParseIntError` through its open `ErrorRaiserComponent` table, and the check asks for
//! `std::io::Error`. The `[CGP-E107]` leaf names the missing path as
//! `@ErrorRaiserComponent.std::io::Error`, a segment `Path!` accepts as written. rustc prints the
//! type qualified because `Error` names several types here, and the tool keeps the qualifier, so the
//! path does not read as the crate's own `backend::Error` or the abstract `Error`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E104 CGP-E107

use core::num::ParseIntError;

use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::DebugError;
use cgp::prelude::*;

pub mod backend {
    #[derive(Debug)]
    pub struct Error(pub String);
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

pub fn unused(_error: backend::Error) {}

fn main() {}
