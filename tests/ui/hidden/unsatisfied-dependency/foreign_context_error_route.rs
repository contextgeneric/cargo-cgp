//! A handler pipeline run on a context defined in another crate, with one error type unrouted.
//!
//! The upstream `App` routes every syntax of its pipeline language, but its open
//! `ErrorRaiserComponent` table has no entry for `IoFailure`, which the `Echo` stage raises.
//! Plain `cargo check` reports two errors about the whole pipeline with rustc's `IsProviderFor`
//! notes, and neither mentions `IoFailure` or `ErrorRaiserComponent`. The tool reads the call as it
//! would on a local context, since `App` carries its own wiring, and reports one `[CGP-E001]`
//! block whose `[CGP-E107]` leaf names `@ErrorRaiserComponent.IoFailure`. Distilled from a
//! pipeline-language program whose context lost an error route.
//!
//! CGP error class: unsatisfied dependency, hidden
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@aux-build: cgp-test-foreign-pipeline
//@quality: acceptable
//@codes: CGP-E001 CGP-E101 CGP-E102 CGP-E104 CGP-E107

use cgp::extra::handler::CanHandle;
use cgp::prelude::*;
use cgp_test_foreign_pipeline::{App, Echo, Pipe, Stdout, backend};

pub async fn run() -> Result<(), backend::Error> {
    App.handle(PhantomData::<Pipe<Product![Echo, Stdout]>>, Vec::new()).await?;
    Ok(())
}

fn main() {}
