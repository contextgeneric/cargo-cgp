//! `DebugError` wired for the `String` key of an `open` error raiser, so it serves itself.
//!
//! `[CGP-E020]` says the lookup leads back to itself, and its help names `DebugError` as the
//! provider that requires `CanRaiseError<String>` again.
//!
//! CGP error class: wiring cycle (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/wiring-cycle.md).

//@quality: acceptable
//@codes: CGP-E020

use core::num::ParseIntError;

use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::DebugError;
use cgp::prelude::*;

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
        ErrorRaiserComponent: ParseIntError,
    }
}

fn main() {}
