#![feature(prelude_import)]
//! A hand-written impl whose own `where` clause names a context defined in another crate, with one
//! error type unrouted.
//!
//! The upstream `App` routes every syntax of its pipeline language, but its open
//! `ErrorRaiserComponent` table has no entry for `IoFailure`, which the `Echo` stage raises. This
//! crate implements `HasEchoRunner` on `Box<App>` where `App` can run the pipeline, a concrete
//! bound the compiler checks at the impl. The tool leaves the error to the text rewrite, which names
//! the `ComposeHandlers` plumbing as the provider at fault and keeps rustc's suggestion to enable
//! `trivial_bounds`, which does not fix the program. The same impl on a local context reports its
//! root cause.
//!
//! CGP error class: unsatisfied dependency, hidden
//! (https://github.com/contextgeneric/cgp-knowledge-base/blob/main/cgp/errors/hidden/unsatisfied-dependency.md).
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
use cgp::extra::handler::CanHandle;
use cgp::prelude::*;
use cgp_test_foreign_pipeline::{App, Echo, Pipe, Stdout};
pub trait HasEchoRunner {}
impl HasEchoRunner for Box<App>
where
    App: CanHandle<Pipe<Product![Echo, Stdout]>, Vec<u8>>,
{}
fn main() {}
