//! A local wrapper trait implemented on a foreign type that holds a context defined in another
//! crate, with one error type unrouted.
//!
//! The upstream `App` routes every syntax of its pipeline language, but its open
//! `ErrorRaiserComponent` table has no entry for `IoFailure`, which the `Echo` stage raises. This
//! crate implements `HasEchoRunner` on every `Box<Context>` whose `Context` can run the pipeline,
//! and the wrapper `CanRunBoxed`, implemented on `Box<App>`, has `HasEchoRunner` as its supertrait.
//! The wrapper-chain anchor descends from the impl of `CanRunBoxed` through that `where` clause to
//! the `CanHandle` consumer on `App`, which it reads as a context since it carries its own wiring,
//! so the impl reports the `[CGP-E113]` missing route a local context would.
//!
//! CGP error class: unsatisfied dependency, hidden
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@aux-build: cgp-test-foreign-pipeline
//@quality: acceptable
//@codes: CGP-E003 CGP-E101 CGP-E102 CGP-E103 CGP-E104 CGP-E113

use cgp::extra::handler::CanHandle;
use cgp::prelude::*;
use cgp_test_foreign_pipeline::{App, Echo, Pipe, Stdout};

pub trait HasEchoRunner {}

impl<Context> HasEchoRunner for Box<Context> where
    Context: CanHandle<Pipe<Product![Echo, Stdout]>, Vec<u8>>
{
}

pub trait CanRunBoxed: HasEchoRunner {}

impl CanRunBoxed for Box<App> {}

fn main() {}
