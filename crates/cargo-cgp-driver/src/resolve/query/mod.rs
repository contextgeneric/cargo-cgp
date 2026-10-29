//! The compiler readers and solver helpers more than one stage uses.
//!
//! Every module here reads the compiler and decides no resolution:
//!
//! - **Solver queries** the walk and the classifiers ask: [`holds`](fn@holds),
//!   [`impl_where_obligations`], [`projection_mismatch`](fn@projection_mismatch),
//!   [`projected_type`](fn@projected_type), and the placeholder folders such as
//!   [`unknowns_to_placeholders`].
//! - **The impl match** every reader of a matched impl shares, [`match_impl`] over
//!   [`fresh_impl_ref`], and the **readers of an impl's clauses:**
//!   [`written_order`](fn@written_order), [`written_order_clauses`], and
//!   [`misdirected_impl_clauses`].
//! - **Renderings of a wiring key**, its path, and its provider: [`describe_key`] and its siblings.
//! - **Syntactic type reads** the call-site anchors and the detectors share: [`lower_hir_ty`] and
//!   [`expr_written_ty`].
//! - **Lookups** several stages make: [`can_use_component_supertrait`], [`impl_self_ty_span`],
//!   [`import_path`](fn@import_path), [`owner_has_impl_of`](fn@owner_has_impl_of), and
//!   [`traits_with_method`](fn@traits_with_method).
//!
//! The resolver's other modules and the class detectors import from here, and nothing here imports
//! them.

mod cgp_item;
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

pub use cgp_item::*;
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
