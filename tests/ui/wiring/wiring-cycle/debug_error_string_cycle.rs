//! `DebugError` wired for the `String` key of an `open` error raiser, so it serves itself.
//!
//! `[CGP-E020]` fires, but its help names the `UseContext` case; neither the headline nor the help
//! names the `DebugError` loop.
//!
//! CGP error class: wiring cycle (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/wiring-cycle.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
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
