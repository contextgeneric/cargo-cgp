//! Recovering the structural mistakes in a wiring or its checks from the generated items the
//! compiler's error lands on.
//!
//! A check table that lists one entry twice makes the compiler reject the two identical check impls
//! it generates, two tables whose check traits share a name collide on that name, and a generic
//! parameter a `for` loop, a `#[default_impl]`, or a wiring entry binds without its key leaves the
//! generated impl unable to bind it; either way the compiler names only the generated items. Each
//! detector here confirms the mistake from the items at the error's caret and fills the rustc-free
//! [`StructuralMistake`](cargo_cgp_error_processing::StructuralMistake) the emitter words.

mod default_impl;
mod duplicate_check;
mod duplicate_check_trait;
mod entry_parameters;
mod loop_key;
mod unit_parameters;

pub use default_impl::*;
pub use duplicate_check::*;
pub use duplicate_check_trait::*;
pub use entry_parameters::*;
pub use loop_key::*;
pub use unit_parameters::*;
