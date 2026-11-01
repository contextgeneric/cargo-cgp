//! Post-processing text transforms for CGP diagnostics.
//!
//! These are the compiler-free string transforms the *driver* applies as its final pass, so raw
//! CGP constructs (`Symbol!` spines, unmet `HasField` bounds) do not look confusing. A diagnostic
//! the tool rewrote gets the whole chain, and one it left alone changes only where a CGP encoding
//! is recovered ([`Origin`]). Each transform is `&str -> Option<String>`, `Some` when it changed
//! the text, `None` when it left it alone, so a caller can tell whether anything matched. They
//! live here, apart from the driver, so they build and are unit-tested on any toolchain without
//! the driver's `rustc_private` linkage. The `chain` module composes the individual transforms in
//! the order they must run.

mod ambiguous;
mod chain;
mod missing_field;
mod resugar_list;
mod resugar_path;
mod resugar_symbol;
mod spine;
mod strip_modules;

pub use ambiguous::*;
pub use chain::*;
pub use missing_field::*;
pub use resugar_list::*;
pub use resugar_path::*;
pub use resugar_symbol::*;
pub use spine::*;
pub use strip_modules::*;
