//! `cargo cgp update`: reinstalling the front-end at the newest version its release channel allows,
//! then running the new binary's `setup`; see
//! `cgp-knowledge-base/cargo-cgp/implementation/distribution/provisioning.md`.

mod index;
mod reinstall;
mod run;
mod select;

pub use index::*;
pub use reinstall::*;
pub use run::*;
pub use select::*;
