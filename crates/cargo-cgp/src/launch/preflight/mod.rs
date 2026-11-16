//! Verifying, before a check, that a matching driver and the pinned toolchain are present, without
//! provisioning anything; see
//! `cgp-knowledge-base/cargo-cgp/implementation/distribution/provisioning.md`.

mod driver_version;
mod evaluate;
mod failure;
mod run;

pub use driver_version::*;
pub use evaluate::*;
pub use failure::*;
pub use run::*;
