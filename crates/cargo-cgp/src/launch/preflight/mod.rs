//! Verifying, before a check, that a matching driver and toolchain are present.
//!
//! `cargo cgp check` never provisions anything itself; it runs this read-only preflight and, on any
//! failure, stops with a message pointing at `cargo cgp setup`. The checks run in order (is the
//! pinned toolchain installed, and does the driver run and match), so a failure of the driver can
//! be told apart from a missing toolchain underneath it. The side-effecting [`run`](fn@run) gathers
//! the facts; the pure [`evaluate`](fn@evaluate) decides on them and [`failure_message`] words each
//! failure, so both are unit-tested without spawning anything.

mod driver_version;
mod evaluate;
mod failure;
mod run;

pub use driver_version::*;
pub use evaluate::*;
pub use failure::*;
pub use run::*;
