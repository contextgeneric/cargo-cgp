//! A wiring value naming a type that does not resolve, beside an `open` dispatch that cycles.
//!
//! The unresolved `backend::Error` makes the error type `{type error}`, which the walk does not
//! descend, so no root cause names it. `@ErrorRaiserComponent.String: DebugError` is a real cycle
//! (`DebugError` raises through `CanRaiseError<String>`), so the overflow is `[CGP-E010]`, beside the
//! `E0433` for `backend`. The `[CGP-E010]` help still names `UseContext`.
//!
//! CGP error class: wiring cycle (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/wiring-cycle.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@quality: usability
//@codes: CGP-E010

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
