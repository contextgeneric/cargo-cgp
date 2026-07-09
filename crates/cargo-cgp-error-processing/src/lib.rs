//! The `cargo-cgp` error-processing stage.
//!
//! This crate is the stateless middle of the [error
//! pipeline](../cargo_cgp/index.html): it takes the structured diagnostics rustc
//! produced and returns a smaller, root-cause-first set of CGP diagnostics. The
//! entrypoint is [`process::process_cgp_errors`]; the output type is
//! [`diagnostic::CgpDiagnostic`]. It is kept free of `rustc_private` so it builds and is
//! tested on any toolchain — see `docs/implementation/error-processing.md` for the design.

pub mod diagnostic;
pub mod preprocess;
pub mod process;

/// Re-export of the diagnostic library the input/output types are built on, so a
/// dependent can name `cargo_metadata::diagnostic::Diagnostic` through this crate.
pub use cargo_metadata;
pub use diagnostic::CgpDiagnostic;
pub use preprocess::{preprocess, resugar_symbol, strip_cgp_prefixes};
pub use process::process_cgp_errors;
