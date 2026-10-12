//! Recovering the structural mistakes in a wiring or its checks from the generated items the
//! compiler's error lands on.
//!
//! A check table that lists one entry twice makes the compiler reject the two identical check impls
//! it generates, naming only the generated check trait. Each detector here confirms the mistake from
//! the items at the error's caret and fills the rustc-free
//! [`StructuralMistake`](cargo_cgp_error_processing::StructuralMistake) the emitter words.

mod duplicate_check;

pub use duplicate_check::*;
