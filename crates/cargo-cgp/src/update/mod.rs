//! `cargo cgp update`: upgrading the tool to the newest version its release channel allows.
//!
//! Update is a thin orchestrator over cargo. It reads the crates.io sparse index for the
//! front-end crate, picks the highest published version **the running version's channel allows**
//! (a stable install moves only to stable releases; a pre-release install moves to any newer
//! version, so a `1.1.0-beta` reaches `1.1.0`), and stops early unless that is strictly newer.
//! Otherwise it reinstalls the front-end at exactly that version with
//! `cargo install --version <version> --locked` and hands off to the freshly installed
//! `cargo cgp setup`, which brings the driver and toolchain up to the new version.
//!
//! There is no self-replacing installer: on Unix `cargo install` atomically replaces the
//! running binary, and on Windows, where the running executable is locked, the reinstall
//! fails and update prints the commands to run by hand instead.
//!
//! Enumerating *all* versions (rather than taking the single "latest" cargo would report) makes
//! channel preservation possible: `cargo search`/`cargo info` only report the max version, which
//! includes pre-releases, so a stable install could not otherwise be offered the latest stable when
//! a higher pre-release also exists. The sparse-index access here follows the shape of the
//! `crates-index` crate but is written directly over the widely-used `ureq` and `serde_json` rather
//! than taking on that dependency.

mod index;
mod reinstall;
mod run;
mod select;

pub use index::*;
pub use reinstall::*;
pub use run::*;
pub use select::*;
