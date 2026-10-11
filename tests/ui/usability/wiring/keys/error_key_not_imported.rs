//! The wiring key `ErrorTypeProviderComponent` used without its import from `cgp::core::error`.
//!
//! Two identical `E0425`s on the key, one with a garbage suggestion; the import is not suggested.
//!
//! CGP error class: none yet; a mistake in what a macro lowers, outside the catalog.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

use cgp::prelude::*;

pub struct App;

delegate_components! {
    App {
        ErrorTypeProviderComponent: UseType<String>,
    }
}

fn main() {}
