//! `raise_error` called by bare name where two traits provide it.
//!
//! rustc's `E0034` suggestions print `cgp::cgp_core::cgp_error::ErrorRaiser` under the tool and omit
//! the provider trait's generics.
//!
//! CGP error class: none; a driver path-printing defect affecting untransformed diagnostics.
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/usability.md.

//@rewrites-falsely: untransformed output prints full paths, because the driver does not set `trimmed_def_paths`

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
