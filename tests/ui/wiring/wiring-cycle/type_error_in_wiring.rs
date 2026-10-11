//! A wiring value naming a type that does not resolve, beside an `open` dispatch that cycles.
//!
//! The unresolved `backend::Error` makes the error type `{type error}`, and the resolver reports
//! `[CGP-E201] the trait bound `{type error}: Debug`` as a root cause, ahead of the `E0433` that is
//! the real mistake.
//!
//! CGP error class: wiring cycle (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/wiring-cycle.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E101 CGP-E102 CGP-E104 CGP-E201

use core::num::ParseIntError;

use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::DebugError;
use cgp::prelude::*;

pub struct App;

delegate_components! {
    App {
        open ErrorRaiserComponent;

        ErrorTypeProviderComponent: UseType<backend::Error>,
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
