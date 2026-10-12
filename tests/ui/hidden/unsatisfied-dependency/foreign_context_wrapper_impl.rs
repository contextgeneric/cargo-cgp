//! A local wrapper trait implemented on a context defined in another crate, with one error type
//! unrouted.
//!
//! The upstream `App` routes every syntax of its pipeline language, but its open
//! `ErrorRaiserComponent` table has no entry for `IoFailure`, which the `Echo` stage raises. This
//! crate asserts the pipeline through a wrapper trait, `CanRunEcho`, whose supertrait is the
//! `CanHandle` consumer trait, and implements it on `App`. The impl-site anchor reads `App` as a
//! context, since it carries its own wiring, so the impl reports the `[CGP-E113]` missing route a
//! local context would. The headline keeps rustc's provider-form bound on the `ComposeHandlers`
//! plumbing, as it does for the same program on a local context.
//!
//! CGP error class: unsatisfied dependency, hidden
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).

//@aux-build: cgp-test-foreign-pipeline
//@quality: acceptable
//@codes: CGP-E002 CGP-E101 CGP-E102 CGP-E103 CGP-E104 CGP-E113

use cgp::extra::handler::CanHandle;
use cgp::prelude::*;
use cgp_test_foreign_pipeline::{App, Echo, Pipe, Stdout};

pub trait CanRunEcho: CanHandle<Pipe<Product![Echo, Stdout]>, Vec<u8>> {}

impl CanRunEcho for App {}

fn main() {}
