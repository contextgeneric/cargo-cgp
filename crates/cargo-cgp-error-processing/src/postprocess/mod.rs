//! Post-processing text transforms for CGP diagnostics.
//!
//! These are the compiler-free string transforms the *driver* applies to a diagnostic it
//! did not rewrite, so raw CGP constructs (`cgp::` path prefixes, `Symbol!` spines, unmet
//! `HasField` bounds) do not look confusing in an error the tool leaves otherwise
//! untouched. Each transform is `&str -> Option<String>` — `Some` when it changed the text,
//! `None` when it left it alone — so a caller can tell whether anything matched. They live
//! here, apart from the driver, so they build and are unit-tested on any toolchain without
//! the driver's `rustc_private` linkage; the driver applies [`postprocess_message`] to each
//! message string of a `DiagInner` on the fallback path. The [chain] module composes the
//! individual transforms in the order they must run.

mod chain;
mod missing_field;
mod resugar_path;
mod resugar_symbol;
mod strip_prefixes;

pub use chain::*;
pub use missing_field::*;
pub use resugar_path::*;
pub use resugar_symbol::*;
pub use strip_prefixes::*;
