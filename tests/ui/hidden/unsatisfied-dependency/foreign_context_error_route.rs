//! A handler pipeline run on a context defined in another crate, with one error type unrouted.
//!
//! The upstream `App` routes every syntax of its pipeline language, but its open
//! `ErrorRaiserComponent` table has no entry for `IoFailure`, which the `Echo` stage raises.
//! Called from this crate, the failure is reported as two `[CGP-E002]` headlines about the
//! whole pipeline with rustc's `IsProviderFor` notes and no tree. Neither the tool's output nor
//! plain `cargo check`'s mentions `IoFailure` or `ErrorRaiserComponent`, so the root cause is
//! absent from the text. The same program with the context defined locally is reshaped into a
//! `[CGP-E107]` leaf naming `@ErrorRaiserComponent.IoFailure`. Distilled from a Hypershell
//! program whose namespace lost its `std::io::Error` route.
//!
//! CGP error class: unsatisfied dependency, hidden
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
//!
//! Issue: cgp-knowledge-base/cargo-cgp/issues/hidden-root-cause.md.

//@aux-build: cgp-test-foreign-pipeline
//@quality: hidden
//@codes: CGP-E002

use cgp::extra::handler::CanHandle;
use cgp::prelude::*;
use cgp_test_foreign_pipeline::{App, Echo, Pipe, Stdout, backend};

pub async fn run() -> Result<(), backend::Error> {
    App.handle(PhantomData::<Pipe<Product![Echo, Stdout]>>, Vec::new()).await?;
    Ok(())
}

fn main() {}
