//! A duplicated `@`-path key whose segment needs its qualifier, in a crate with an `Error` of its
//! own.
//!
//! `std::io::Error` beside a local `Error` is printed qualified, since the bare name is ambiguous,
//! and the message keeps the qualifier, so the key reads as its entry wrote it and names the type
//! it wires. A segment cut to `Error` would name the crate's own type.
//!
//! CGP error class:
//! <https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/wiring/conflicting-wiring.md>.

//@quality: acceptable
//@codes: CGP-E010

use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::{DebugError, RaiseFrom};
use cgp::prelude::*;

#[derive(Debug)]
pub struct Error;

pub struct App;

delegate_components! {
    App {
        open ErrorRaiserComponent;

        ErrorTypeProviderComponent: UseType<String>,
        @ErrorRaiserComponent.std::io::Error: DebugError,
        @ErrorRaiserComponent.std::io::Error: RaiseFrom,
        @ErrorRaiserComponent.Error: DebugError,
    }
}

fn main() {}
