//! Renaming CGP wiring messages to name the traits behind a component marker.
//!
//! This module lives in the rustc-free error-processing crate on purpose. The rewrite is a
//! plain string-to-string transform over a [`ComponentNameMap`], so it is unit-tested on any
//! toolchain without a `TyCtxt`. The compiler-coupled half — walking the trait graph to *build*
//! the map — lives in the driver (`cargo-cgp-driver`), which hands the result in through the
//! map's `fn`-pointer initializer. It is used by the driver's diagnostic emitter, alongside the
//! [postprocess](crate::postprocess) fallback transforms; the [message] module documents the
//! rewrite forms in full.

mod message;
mod names;
mod parse;
mod text;

pub use message::*;
pub use names::*;
pub use parse::*;
