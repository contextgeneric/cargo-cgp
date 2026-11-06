#![feature(prelude_import)]
//! A local wrapper trait implemented on a context defined in another crate, with one error type
//! unrouted.
//!
//! The upstream `App` routes every syntax of its pipeline language, but its open
//! `ErrorRaiserComponent` table has no entry for `IoFailure`, which the `Echo` stage raises. This
//! crate asserts the pipeline through a wrapper trait, `CanRunEcho`, whose supertrait is the
//! `CanHandle` consumer trait, and implements it on `App`. The impl-site anchor reads `App` as a
//! context, since it carries its own wiring, so the impl reports the missing route a local context
//! would: a `[CGP-E003]` headline reading
//! `` missing wiring for `@ErrorRaiserComponent.IoFailure` on `App` ``.
//!
//! CGP error class: unsatisfied dependency, hidden
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::extra::handler::CanHandle;
use cgp::prelude::*;
use cgp_test_foreign_pipeline::{App, Echo, Pipe, Stdout};
pub trait CanRunEcho: CanHandle<Pipe<Product![Echo, Stdout]>, Vec<u8>> {}
impl CanRunEcho for App {}
fn main() {}
