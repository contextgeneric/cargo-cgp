//! An `open` error table missing the route for a dependency's error, in a crate that glob-imports
//! an error of the same name from a module named after that dependency.
//!
//! `App` routes `ParseIntError` through its open `ErrorRaiserComponent` table, and the check asks for
//! `cgp_test_foreign_pipeline::IoFailure`, an error the dependency defines at its root. The crate
//! glob-imports a prelude that re-exports `backend::cgp_test_foreign_pipeline::IoFailure`, so the
//! bare name `IoFailure` in this crate means that item, whose path ends in the dependency's path.
//! rustc prints the dependency's error from its crate name, so the tool keeps the qualifier: the
//! headline reads `CanRaiseError<cgp_test_foreign_pipeline::IoFailure>` and the leaf
//! `@ErrorRaiserComponent.cgp_test_foreign_pipeline::IoFailure`.
//!
//! CGP error class: check-trait failure (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/checks/check-trait-failure.md).

//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E103 CGP-E113
//@aux-build: cgp-test-foreign-pipeline

use core::num::ParseIntError;

use cgp::core::error::{ErrorRaiserComponent, ErrorTypeProviderComponent};
use cgp::extra::error::DebugError;
use prelude::*;

pub mod backend {
    pub mod cgp_test_foreign_pipeline {
        #[derive(Debug)]
        pub struct IoFailure;
    }
}

pub mod prelude {
    pub use cgp::prelude::*;

    pub use crate::backend::cgp_test_foreign_pipeline::*;
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
        ErrorRaiserComponent: cgp_test_foreign_pipeline::IoFailure,
    }
}

pub fn unused(_error: IoFailure) {}

fn main() {}
