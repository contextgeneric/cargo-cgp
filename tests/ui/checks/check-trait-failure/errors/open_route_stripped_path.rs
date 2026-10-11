//! An `open` error table missing the route for `std::io::Error`, in a crate with its own `Error`.
//!
//! `App` routes `ParseIntError` through its open `ErrorRaiserComponent` table, and the check asks for
//! `std::io::Error`. The `[CGP-E107]` leaf names the missing path, but strips the type segment to
//! `@ErrorRaiserComponent.Error`, which reads as the crate's own `backend::Error` or the abstract
//! `Error` rather than `std::io::Error`. rustc prints the path qualified because `Error` names
//! several types here.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: usability
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
