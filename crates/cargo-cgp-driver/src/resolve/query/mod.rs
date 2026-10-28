//! The compiler readers and solver helpers more than one stage uses.
//!
//! Every module here reads the compiler and decides no resolution:
//!
//! - **Solver queries** the walk and the classifiers ask: [`holds`](mod@holds),
//!   [`impl_match`], [`projection_mismatch`](mod@projection_mismatch),
//!   [`projected_type`](mod@projected_type), and the placeholder folders in [`unknowns`].
//! - **Readers of an impl's clauses:** [`written_order`](mod@written_order) and [`misdirected`].
//! - **Renderings of a wiring key**, its path, and its provider: [`keys`].
//! - **Syntactic type reads** the call-site anchors and the detectors share: [`lower`] and
//!   [`written_ty`].
//! - **Lookups** several stages make: [`check_impl`], [`import_path`](mod@import_path),
//!   [`owner_has_impl_of`](mod@owner_has_impl_of), and
//!   [`traits_with_method`](mod@traits_with_method).
//!
//! The resolver's other modules and the class detectors import from here, and nothing here imports
//! them.

mod check_impl;
mod holds;
mod impl_match;
mod import_path;
mod keys;
mod lower;
mod misdirected;
mod owner_has_impl_of;
mod projected_type;
mod projection_mismatch;
mod traits_with_method;
mod unknowns;
mod written_order;
mod written_ty;

pub use check_impl::*;
pub use holds::*;
pub use impl_match::*;
pub use import_path::*;
pub use keys::*;
pub use lower::*;
pub use misdirected::*;
pub use owner_has_impl_of::*;
pub use projected_type::*;
pub use projection_mismatch::*;
pub use traits_with_method::*;
pub use unknowns::*;
pub use written_order::*;
pub use written_ty::*;
