//! The `cargo-cgp` error-processing stage.
//!
//! This crate is the stateless middle of the [error
//! pipeline](../cargo_cgp/index.html): it takes the structured diagnostics rustc
//! produced and returns a smaller, root-cause-first set of CGP diagnostics. The
//! entrypoint is [`process::process_cgp_errors`]; the output type is
//! [`diagnostic::CgpDiagnostic`]. It is kept free of `rustc_private` so it builds and is
//! tested on any toolchain — see `docs/implementation/error-processing.md` for the design.
//!
//! Being rustc-free, this crate is also the home of the [`rewrite`] module — the pure
//! string transform that renames CGP wiring messages. That logic is driven by the *driver*
//! (`cargo-cgp-driver`), not by [`process_cgp_errors`], but it lives here so it can be
//! unit-tested without the driver's compiler linkage; the driver supplies the
//! compiler-derived name map through [`rewrite::ComponentNameMap`].

pub mod code;
pub mod diagnostic;
pub mod preprocess;
pub mod process;
pub mod rewrite;
pub mod tree;

/// Re-export of the diagnostic library the input/output types are built on, so a
/// dependent can name `cargo_metadata::diagnostic::Diagnostic` through this crate.
pub use cargo_metadata;
pub use diagnostic::{CgpDiagnostic, CgpDiagnosticDetail};
pub use preprocess::{extract_missing_fields, preprocess, resugar_symbol, strip_cgp_prefixes};
pub use process::process_cgp_errors;
pub use rewrite::{ComponentNameMap, ComponentTraitNames, rewrite_message};
pub use tree::{DependencyTree, render_dependency_tree};
