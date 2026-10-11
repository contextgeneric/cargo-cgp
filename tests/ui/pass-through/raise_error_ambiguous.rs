//! `raise_error` called by bare name where two traits provide it.
//!
//! The error is plain Rust: the call is ambiguous (`E0034`). The tool must print what plain
//! `cargo check` prints, keeping the `cgp::prelude::CanRaiseError` path rustc prints in a note.
//!
//! CGP error class: none; plain Rust the tool must pass through unchanged.

use cgp::core::error::{ErrorRaiser, ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::RaiseFrom;
use cgp::prelude::*;

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
        ErrorRaiserComponent: RaiseFrom,
    }
}

fn main() {
    let _ = App::raise_error(String::new());
}
